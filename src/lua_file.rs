use crate::config::{ListArgs, NuMakeArgs};
use crate::error::{to_lua_result, NUMAKE_ERROR};
use crate::target::Target;
use anyhow::anyhow;
use mlua::{Compiler, FromLua, Lua, UserData, UserDataFields, UserDataMethods, Value};
use std::collections::HashMap;

use std::fs;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use tempfile::tempfile;
use uuid::Uuid;
use zip::ZipArchive;

#[derive(Clone)]
pub struct LuaFile {
    targets: HashMap<String, Target>,

    file: PathBuf,
    workspace: PathBuf,
    workdir: PathBuf, // Should already exist

    target: String,
    output: Option<String>,

    toolset_compiler: Option<String>,
    toolset_linker: Option<String>,

    arguments: Vec<String>,
    msvc: bool,

    lua_compiler: Compiler,
}

impl UserData for LuaFile {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("arguments", |_, this| Ok(this.arguments.clone()));
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("create_target", |_, this, name: String| {
            to_lua_result(Target::new(
                name.clone(),
                this.toolset_compiler.clone(),
                this.toolset_linker.clone(),
                this.msvc.clone(),
                this.output.clone(),
                this.workdir.clone(),
            ))
        });

        methods.add_method_mut("register_target", |_, this, target: Target| {
            Ok(this.targets.insert(target.name.clone(), target))
        });

        methods.add_method("download_zip", |_, this, url: String| {
            to_lua_result(this.workspace_download_zip(url))
        });

        methods.add_method("require_url", |lua, this, url: String| {
            to_lua_result(this.require_url(lua, url))
        });
    }
}

impl<'lua> FromLua<'lua> for LuaFile {
    fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(user_data) => Ok(user_data.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

impl LuaFile {
    pub fn new(args: &NuMakeArgs) -> anyhow::Result<Self> {
        Ok(LuaFile {
            targets: HashMap::new(),
            workdir: dunce::canonicalize(&args.workdir)?,
            file: dunce::canonicalize(dunce::canonicalize(&args.workdir)?.join(&args.file))?,
            workspace: dunce::canonicalize(&args.workdir)?.join(".numake"),
            target: args.target.clone(),
            toolset_compiler: args.toolset_compiler.clone(),
            toolset_linker: args.toolset_linker.clone(),
            output: args.output.clone(),

            arguments: args.arguments.clone().unwrap_or(vec![]),
            msvc: args.msvc.clone(),

            lua_compiler: Compiler::new().set_debug_level(2).set_coverage_level(2),
        })
    }

    pub fn new_dummy(args: &ListArgs) -> anyhow::Result<Self> {
        Ok(LuaFile {
            targets: HashMap::new(),
            workdir: dunce::canonicalize(&args.workdir)?,
            file: dunce::canonicalize(dunce::canonicalize(&args.workdir)?.join(&args.file))?,
            workspace: dunce::canonicalize(&args.workdir)?.join(".numake"),
            target: "".to_string(),
            toolset_compiler: None,
            toolset_linker: None,
            output: None,

            arguments: vec![],
            msvc: false,

            lua_compiler: Compiler::new().set_debug_level(2).set_coverage_level(2),
        })
    }

    pub fn process(&mut self, lua_state: &Lua) -> anyhow::Result<()> {
        lua_state.set_compiler(self.lua_compiler.clone());

        if !self.workspace.exists() {
            fs::create_dir_all(&self.workspace)?;
        }

        if !self.file.starts_with(&self.workdir) {
            // Throw error if file is outside working directory
            Err(anyhow!(&NUMAKE_ERROR.PATH_OUTSIDE_WORKING_DIR))?
        }

        lua_state.globals().set("workspace", self.clone())?;

        let file_uuid = Uuid::new_v8(
            *self
                .file
                .to_str()
                .unwrap()
                .as_bytes()
                .last_chunk::<16>()
                .unwrap(),
        )
        .to_string();

        let cache_dir = self.workspace.join("cache");
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        let cache_toml = cache_dir.join("cache.toml");
        let file_size = self.file.metadata()?.len().to_string();
        let file_size_toml = toml::Value::from(file_size.clone());
        if !cache_toml.exists() {
            fs::write(&cache_toml, format!("{}=\"{}\"", &file_uuid, file_size))?;
        }

        let mut table = toml::Table::from_str(&*fs::read_to_string(&cache_toml)?)?;
        let file_cache = cache_dir.join(format!("{}.{}", &file_uuid, "nucache"));
        if table[&file_uuid] == file_size_toml && file_cache.exists() {
            lua_state
                .load(fs::read(&file_cache)?)
                .set_name(self.file.file_name().unwrap().to_str().unwrap())
                .exec()?;
        } else if table[&file_uuid] != file_size_toml || !file_cache.exists() {
            let file_content = fs::read_to_string(&self.file)?;
            let result = lua_state
                .load(&file_content)
                .set_name(self.file.file_name().unwrap().to_str().unwrap())
                .exec();
            if result.is_ok() {
                fs::write(&file_cache, self.lua_compiler.compile(&file_content))?;
                table[&file_uuid] = file_size_toml;
                fs::write(&cache_toml, table.to_string())?;
            } else {
                Err(result.err().unwrap())?;
            }
        }

        let lua_workspace: Self = lua_state.globals().get("workspace")?;
        self.targets = lua_workspace.targets.clone();

        Ok(())
    }

    pub fn list_targets(&self) -> anyhow::Result<String> {
        Ok(self
            .targets
            .iter()
            .map(|(name, _)| name.clone() + " ")
            .collect())
    }

    pub fn build(&mut self) -> anyhow::Result<()> {
        if self.target == "all" || self.target == "*" {
            for (target, _) in self.targets.clone() {
                self.build_target(&target)?;
            }
            Ok(())
        } else {
            self.build_target(&self.target.clone())
        }
    }

    fn build_target(&self, _target: &String) -> anyhow::Result<()> {
        if self.targets.get(_target).is_none() {
            Err(anyhow!(&NUMAKE_ERROR.TARGET_NOT_FOUND))?
        }

        let target_obj = self.targets.get(&self.target).unwrap();

        let obj_dir: PathBuf = self.workspace.join(format!("obj/{}", &self.target));
        let out_dir: PathBuf = self.workspace.join(format!("out/{}", &self.target));

        if !obj_dir.exists() {
            fs::create_dir_all(&obj_dir)?;
        }

        if !out_dir.exists() {
            fs::create_dir_all(&out_dir)?;
        }

        let mut o_files: Vec<String> = Vec::new(); // Can't assume all compilers support wildcards.

        let toolset_compiler: Option<String> = if self.toolset_compiler.is_none() {
            target_obj.toolset_compiler.clone()
        } else {
            self.toolset_compiler.clone()
        };
        let toolset_linker: Option<String> = if self.toolset_linker.is_none() {
            target_obj.toolset_linker.clone()
        } else {
            self.toolset_linker.clone()
        };
        let output: Option<String> = if self.output.is_none() {
            target_obj.output.clone()
        } else {
            self.output.clone()
        };

        for file in target_obj.files.clone() {
            let mut compiler = Command::new(toolset_compiler.clone().unwrap_or("NULL".to_string()));

            let o_file = format!(
                "{}/{}.{}",
                &obj_dir.to_str().unwrap_or("ERROR"),
                &file
                    .file_name()
                    .unwrap_or("ERROR".as_ref())
                    .to_str()
                    .unwrap_or("ERROR"),
                if target_obj.msvc { "obj" } else { "o" }
            );

            let mut compiler_args = Vec::from([
                "-c".to_string(),
                if target_obj.msvc {
                    format!("-Fo:{}", &o_file)
                } else {
                    format!("-o{}", &o_file)
                },
            ]);

            o_files.push(o_file);

            for incl in target_obj.include_paths.clone() {
                compiler_args.push(format!("-I{incl}"))
            }

            for define in target_obj.defines.clone() {
                compiler_args.push(format!("-D{define}"))
            }

            for flag in target_obj.compiler_flags.clone() {
                compiler_args.push(flag)
            }

            compiler_args.push(file.to_str().unwrap_or("ERROR").to_string());

            println!(
                "\n{} exited with {}.\n",
                toolset_compiler.clone().unwrap_or("NULL".to_string()),
                compiler
                    .args(&compiler_args)
                    .current_dir(&self.workdir)
                    .status()?
            );
        }

        let mut linker = Command::new(toolset_linker.clone().unwrap_or("NULL".to_string()));
        let mut linker_args = Vec::new();

        linker_args.append(&mut o_files);

        if !target_obj.msvc {
            for path in target_obj.lib_paths.clone() {
                linker_args.push(format!("-L{path}"))
            }

            for lib in target_obj.libs.clone() {
                linker_args.push(format!("-l{lib}"))
            }

            linker_args.push(format!(
                "-o{}/{}",
                &out_dir.to_str().unwrap_or("ERROR"),
                &output.unwrap_or("out".to_string())
            ));
        } else {
            linker_args.append(&mut target_obj.libs.clone());

            linker_args.push("/link".to_string());

            linker_args.push(format!(
                "/out:{}/{}",
                &out_dir.to_str().unwrap_or("ERROR"),
                &output.unwrap_or("out".to_string())
            ));

            for path in target_obj.lib_paths.clone() {
                linker_args.push(format!("/LIBPATH:{path}"))
            }
        }

        for flag in target_obj.linker_flags.clone() {
            linker_args.push(flag)
        }

        println!(
            "\n{} exited with {}. \n",
            toolset_linker.clone().unwrap_or("NULL".to_string()),
            linker
                .args(&linker_args)
                .current_dir(&self.workdir)
                .status()?
        );

        for (oldpath, newpath) in target_obj.assets.clone() {
            let old_path = PathBuf::from(&oldpath); // Already canonicalized and validated.
            let new_path = out_dir.join(&newpath); // Needs to be validated during build, and so we do.

            if new_path.starts_with(&out_dir) {
                // Make sure we haven't escaped our output dir
                fs::copy(old_path, new_path)?;
            } else {
                Err(anyhow!(format!(
                    "Asset file '{}' copied to invalid destination! ({})",
                    old_path.to_str().unwrap_or("ERROR"),
                    new_path.to_str().unwrap_or("ERROR")
                )))?
            }
        }

        Ok(())
    }

    fn require_url(&self, lua_state: &Lua, url: String) -> anyhow::Result<()> {
        let file_uuid = Uuid::new_v8(*url.as_bytes().last_chunk::<16>().unwrap()).to_string();
        let cache_dir = self.workspace.join("cache");
        let cache_toml = cache_dir.join("cache.toml");
        if !cache_toml.exists() {
            fs::write(&cache_toml, format!("{}=\"-1\"", &file_uuid))?;
        }

        let mut table = toml::Table::from_str(&*fs::read_to_string(&cache_toml)?)?;
        let file_cache = cache_dir.join(format!("{}.{}", &file_uuid, "nucache"));

        let response = reqwest::blocking::get(&url)?;
        if !response.status().is_success() {
            if table.contains_key(&file_uuid) && file_cache.exists() {
                Ok(lua_state
                    .load(fs::read(&file_cache)?)
                    .set_name(&url)
                    .exec()?)
            } else {
                Err(anyhow!(response.status()))?
            }
        } else {
            let file_size = response.content_length().unwrap_or(0).to_string();
            let file_size_toml = toml::Value::from(file_size.clone());
            if !table.contains_key(&file_uuid) {
                table.insert(file_uuid.clone(), toml::Value::from("-1"));
            }

            if table[&file_uuid] == file_size_toml && file_cache.exists() {
                Ok(lua_state
                    .load(fs::read(&file_cache)?)
                    .set_name(&url)
                    .exec()?)
            } else if table[&file_uuid] != file_size_toml || !file_cache.exists() {
                let file_content = response.text()?;
                let result = lua_state.load(&file_content).set_name(&url).eval();
                if result.is_ok() {
                    fs::write(&file_cache, self.lua_compiler.compile(&file_content))?;
                    table[&file_uuid] = file_size_toml;
                    fs::write(&cache_toml, table.to_string())?;
                    Ok(result.ok().unwrap())
                } else {
                    Err(result.err().unwrap())?
                }
            } else {
                Err(anyhow!("URL REQUIRE ERROR"))?
            }
        }
    }

    fn workspace_download_zip(&self, url: String) -> anyhow::Result<String> {
        let response = reqwest::blocking::get(&url);
        if response.is_err() {
            Err(response.err().unwrap())? // Convert error to mlua error
        } else {
            let ok_response = response.ok().unwrap();
            if ok_response.status().is_success() {
                let buf: [u8; 16] = *url.as_bytes().last_chunk::<16>().unwrap();
                let path = format!(
                    // Where the archive will be extracted.
                    "{}/remote/{}",
                    self.workspace.to_str().unwrap_or("ERROR"),
                    Uuid::new_v8(buf)
                );

                if fs::metadata(&path).is_err() {
                    // Don't "download" again. (data already in memory)
                    let bytes = ok_response.bytes();
                    fs::create_dir_all(&path)?;
                    let path_buf = dunce::canonicalize(&path)?;
                    let mut tempfile = tempfile()?; // Create a tempfile as a buffer for our response bytes because nothing else implements Seek ffs
                    tempfile.write_all(bytes.unwrap().as_ref())?;
                    ZipArchive::new(BufReader::new(tempfile))?.extract(&path_buf)?;
                }
                Ok(path) // Return path if already exists
            } else {
                Err(anyhow!(ok_response.status()))?
            }
        }
    }
}

use std::{fs};

use std::io::{BufReader, Write};


use std::path::{PathBuf};
use std::process::Command;
use std::ptr::null_mut;
use anyhow::anyhow;


use mlua::Lua;
use mlua::prelude::LuaError;
use tempfile::tempfile;
use uuid::Uuid;
use zip::ZipArchive;

use crate::config::NuMakeArgs;
pub struct Project {
    pub lua_instance: Lua,

    pub compiler_flags: Vec<String>,
    pub linker_flags: Vec<String>,
    pub include_paths: Vec<String>,
    pub lib_paths: Vec<String>,
    pub libs: Vec<String>,
    pub files: Vec<String>,
    pub defines: Vec<String>,

    pub assets: Vec<(String, String)>,

    pub lang: String,
    pub output: String,

    pub file: PathBuf,
    pub workspace: PathBuf,
    pub workdir: PathBuf, // Should already exist

    pub target: Option<String>,
    pub configuration: Option<String>,
    pub arch: Option<String>,
    pub toolset_compiler: Option<String>,
    pub toolset_linker: Option<String>,

    pub msvc: bool,
}

static mut PTR: *mut Project = null_mut();

impl Project {
    pub fn new(args: &NuMakeArgs) -> Self {
        Project {
            lua_instance: Lua::new(),

            compiler_flags: Vec::new(),
            linker_flags: Vec::new(),
            include_paths: Vec::new(),
            lib_paths: Vec::new(),
            libs: Vec::new(),
            lang: String::new(),
            files: Vec::new(),
            defines: Vec::new(),
            assets: Vec::new(),

            target: args.target.clone(),
            configuration: args.configuration.clone(),
            arch: args.arch.clone(),
            toolset_compiler: args.toolset_compiler.clone(),
            toolset_linker: args.toolset_linker.clone(),
            output: args.output.clone(),

            file: dunce::canonicalize(&args.file).unwrap(),
            workdir: dunce::canonicalize(&args.workdir).unwrap(),
            workspace: dunce::canonicalize(&args.workdir).unwrap().join(".numake"),

            msvc: args.msvc,
        }
    }

    pub fn setup_lua_vals(&mut self) -> anyhow::Result<()> {
        unsafe {
            PTR = self;
        }

        self.lua_instance
            .globals()
            .set("msvc", self.msvc.clone())?;

        self.lua_instance
            .globals()
            .set("target", self.target.clone())?;

        self.lua_instance
            .globals()
            .set("configuration", self.configuration.clone())?;

        self.lua_instance
            .globals()
            .set("arch", self.arch.clone())?;

        self.lua_instance
            .globals()
            .set("toolset_compiler", self.toolset_compiler.clone())?;

        self.lua_instance
            .globals()
            .set("toolset_linker", self.toolset_linker.clone())?;

        self.lua_instance
            .globals()
            .set("output", self.output.clone())?;

        // Functions
        self.lua_instance
            .globals()
            .set(
                "add_include_path",
                self.lua_instance
                    .create_function_mut(|_, path: String| unsafe {
                        (*PTR).add_include_path(path.clone())
                    })?,
            )?;

        self.lua_instance
            .globals()
            .set(
                "set_include_paths",
                self.lua_instance
                    .create_function_mut(|_, paths: Vec<String>| unsafe {
                        (*PTR).set_include_paths(paths)
                    })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "add_compiler_flag",
                self.lua_instance
                    .create_function_mut(|_, flag: String| unsafe {
                        (*PTR).add_compiler_flag(flag)
                    })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "set_compiler_flags",
                self.lua_instance
                    .create_function_mut(|_, flags: Vec<String>| unsafe {
                        (*PTR).set_compiler_flags(flags)
                    })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "add_linker_flag",
                self.lua_instance
                    .create_function_mut(|_, flag: String| unsafe { (*PTR).add_linker_flag(flag) })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "set_linker_flags",
                self.lua_instance
                    .create_function_mut(|_, flags: Vec<String>| unsafe {
                        (*PTR).set_linker_flags(flags)
                    })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "add_lib_path",
                self.lua_instance
                    .create_function_mut(|_, path: String| unsafe { (*PTR).add_lib_path(path) })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "set_lib_paths",
                self.lua_instance
                    .create_function_mut(|_, paths: Vec<String>| unsafe {
                        (*PTR).set_lib_paths(paths)
                    })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "add_lib",
                self.lua_instance
                    .create_function_mut(|_, lib: String| unsafe { (*PTR).add_lib(lib) })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "set_libs",
                self.lua_instance
                    .create_function_mut(|_, libs: Vec<String>| unsafe { (*PTR).set_libs(libs) })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "add_dir",
                self.lua_instance
                    .create_function_mut(|_, (path, recursive): (String, bool)| unsafe {
                        (*PTR).add_dir(
                            &dunce::canonicalize((*PTR).workdir.join(path)).unwrap(),
                            recursive,
                        )
                    })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "add_file",
                self.lua_instance
                    .create_function_mut(|_, path: String| unsafe {
                        (*PTR).add_file(&dunce::canonicalize((*PTR).workdir.join(path)).unwrap())
                    })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "add_asset",
                self.lua_instance
                    .create_function_mut(|_, (filepath, newpath): (String, String)| unsafe {
                        (*PTR).add_asset(filepath, newpath)
                    })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "define",
                self.lua_instance
                    .create_function_mut(|_, define: String| unsafe { (*PTR).define(define) })
                    ?,
            )
            ?;

        self.lua_instance
            .globals()
            .set(
                "workspace_download_zip",
                self.lua_instance
                    .create_function_mut(|_, url: String| unsafe {
                        Ok((*PTR).workspace_download_zip(url).unwrap())
                    })
                    ?,
            )
            ?;

        // Get and require an online numake script, for package management mainly.
        self.lua_instance
            .globals()
            .set(
                "require_url",
                self.lua_instance
                    .create_function_mut(|_, url: String| unsafe { (*PTR).require_url(url) })
                    ?,
            )?;

        Ok(())
    }

    pub fn process(&mut self) -> anyhow::Result<()>{
        if !self.workspace.exists() {
            fs::create_dir_all(&self.workspace)?;
        }

        let filepath = dunce::canonicalize(
            self.workdir.join(&self.file)
        )?;

        if !filepath.starts_with(&self.workdir) { // Throw error if file is outside working directory
            Err(anyhow!("NuMake file cannot be outside the working directory!"))?
        }

        // Parse file
        self.lua_instance
            .load(fs::read_to_string(filepath)?)
            .exec()
            .unwrap();

        self.toolset_compiler = self.lua_instance.globals().get("toolset_compiler").ok();
        self.toolset_linker = self.lua_instance.globals().get("toolset_linker").ok();
        self.target = self.lua_instance.globals().get("target")?;
        self.arch = self.lua_instance.globals().get("arch")?;
        self.configuration = self.lua_instance.globals().get("configuration")?;

        self.output = self.lua_instance.globals().get("output")?;
        self.msvc = self.lua_instance.globals().get("msvc")?;

        Ok(())
    }

    pub fn build(&mut self) -> anyhow::Result<()> {
        if self.toolset_compiler == None {
            Err(anyhow!("ERROR: NO COMPILER SPECIFIED!"))?
        }

        if self.toolset_linker == None {
            Err(anyhow!("ERROR: NO LINKER SPECIFIED!"))?
        }

        let config: String = format!(
            "{}-{}-{}",
            self.arch.clone().unwrap_or("null".to_string()),
            self.target.clone().unwrap_or("null".to_string()),
            self.configuration.clone().unwrap_or("null".to_string())
        );

        let obj_dir: PathBuf = self.workspace.join(format!("obj/{}", &config));
        let out_dir: PathBuf = self.workspace.join(format!("out/{}", &config));

        if !obj_dir.exists() {
            fs::create_dir_all(&obj_dir)?;
        }

        if !out_dir.exists() {
            fs::create_dir_all(&out_dir)?;
        }

        let mut o_files: Vec<String> = Vec::new(); // Can't assume all compilers support wildcards.

        for file in self.files.clone() {
            let mut compiler = Command::new(&self.toolset_compiler.clone().unwrap_or("null".to_string()));

            let o_file = format!(
                "{}/{}.{}",
                &obj_dir.to_str().unwrap_or("ERROR"),
                &file,
                if self.msvc { "obj" } else { "o" }
            );

            let mut compiler_args = Vec::from([
                "-c".to_string(),
                if self.msvc {
                    format!("-Fo:{}", &o_file)
                } else {
                    format!("-o{}", &o_file)
                },
            ]);

            o_files.push(o_file);

            for incl in self.include_paths.clone() {
                compiler_args.push(format!("-I{incl}"))
            }

            for define in self.defines.clone() {
                compiler_args.push(format!("-D{define}"))
            }

            for flag in self.compiler_flags.clone() {
                compiler_args.push(flag)
            }

            compiler_args.push(file);

            println!(
                "{} exited with {}.",
                self.toolset_compiler.clone().unwrap_or("null".to_string()),
                compiler
                    .args(&compiler_args)
                    .current_dir(&self.workdir)
                    .status()?
            );
        }

        let mut linker = Command::new(&self.toolset_linker.clone().unwrap_or("null".to_string()));
        let mut linker_args = Vec::from(o_files);

        for lib in self.libs.clone() {
            linker_args.push(if self.msvc { lib } else { format!("-l{lib}") })
        }

        if self.msvc {
            linker_args.push("/link".to_string());
        }

        if self.msvc {
            linker_args.push(format!(
                "/out:{}/{}",
                &obj_dir.to_str().unwrap_or("ERROR"),
                &self.output
            ));
        } else {
            linker_args.push(format!("-o{}/{}", &obj_dir.to_str().unwrap_or("ERROR"), &self.output));
        }

        for flag in self.linker_flags.clone() {
            linker_args.push(flag)
        }

        for path in self.lib_paths.clone() {
            linker_args.push(if self.msvc {
                format!("/LIBPATH:{path}")
            } else {
                format!("-L{path}")
            })
        }

        println!(
            "{} exited with {}. ",
            self.toolset_linker.clone().unwrap(),
            linker
                .args(&linker_args)
                .current_dir(&self.workdir)
                .status()?
        );

        for (oldpath, newpath) in self.assets.clone() {
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

    fn workspace_download_zip(&mut self, url: String) -> anyhow::Result<String> {
        let response = reqwest::blocking::get(&url).unwrap();

        if response.status().is_success() {

            let buf: [u8; 16] = *url.as_bytes().last_chunk::<16>().unwrap();
            let path = format!( // Where the archive will be extracted.
                "{}/remote/{}",
                self.workspace.to_str().unwrap_or("ERROR"),
                Uuid::new_v8(buf)
            );
            if fs::metadata(&path).is_err() {
                // Don't "download" again. (data already in memory)
                fs::create_dir_all(&path)?;
                let path_buf = dunce::canonicalize(&path)?;
                let mut tempfile = tempfile()?; // Create a tempfile as a buffer for our response bytes because nothing else implements Seek ffs
                tempfile.write_all(response.bytes().unwrap().as_ref())?;

                let mut zip = ZipArchive::new(BufReader::new(tempfile))?;
                zip.extract(&path_buf)?;
            }
            Ok(path)
        } else {
            Ok(response.status().to_string())
        }
    }

    fn require_url(&mut self, url: String) -> mlua::Result<i32> {
        let response = reqwest::blocking::get(url).unwrap();
        self.lua_instance
            .load(response.text().unwrap().to_string())
            .exec()
            .unwrap();
        Ok(0)
    }

    fn add_include_path(&mut self, path: String) -> mlua::Result<i32> {
        self.include_paths.push(path);
        Ok(0)
    }

    fn set_include_paths(&mut self, paths: Vec<String>) -> mlua::Result<i32> {
        self.include_paths = paths.to_vec();
        Ok(0)
    }

    fn add_compiler_flag(&mut self, flag: String) -> mlua::Result<i32> {
        self.compiler_flags.push(flag);
        Ok(0)
    }

    fn set_compiler_flags(&mut self, flags: Vec<String>) -> mlua::Result<i32> {
        self.compiler_flags = flags;
        Ok(0)
    }

    fn add_linker_flag(&mut self, flag: String) -> mlua::Result<i32> {
        self.linker_flags.push(flag);
        Ok(0)
    }

    fn set_linker_flags(&mut self, flags: Vec<String>) -> mlua::Result<i32> {
        self.linker_flags = flags;
        Ok(0)
    }

    fn add_lib_path(&mut self, path: String) -> mlua::Result<i32> {
        self.lib_paths.push(path);
        Ok(0)
    }

    fn set_lib_paths(&mut self, paths: Vec<String>) -> mlua::Result<i32> {
        self.lib_paths = paths.to_vec();
        Ok(0)
    }

    fn add_lib(&mut self, lib: String) -> mlua::Result<i32> {
        self.libs.push(lib);
        Ok(0)
    }

    fn set_libs(&mut self, libs: Vec<String>) -> mlua::Result<i32> {
        self.libs = libs;
        Ok(0)
    }

    fn add_dir(&mut self, pathbuf: &PathBuf, recursive: bool) -> mlua::Result<()> {
        if !pathbuf.starts_with(&self.workdir) {
            Err(LuaError::runtime("Path may not exit working directory!"))?
        }

        for entry in fs::read_dir(pathbuf)? {
            let path = dunce::canonicalize(entry?.path())?;
            if path.is_dir() && recursive {
                self.add_dir(&path, true)?;
            }
            if path.is_file() {
                self.add_file(&path)?;
            }
        }
        Ok(())
    }

    fn add_file(&mut self, file: &PathBuf) -> mlua::Result<()> {
        if !file.starts_with(&self.workdir) {
            Err(LuaError::runtime("Path may not exit working directory!"))?
        }

        if file.exists() && file.is_file()
        {
            self.files.push(file.to_str().unwrap_or("ERROR").to_string());
            Ok(())
        } else {
            Err(LuaError::runtime("Invalid file!"))?
        }
    }

    fn add_asset(&mut self, filepath: String, newpath: String) -> mlua::Result<()> {
        let path = &dunce::canonicalize(self.workdir.join(&filepath))?; // Will automatically error if path doesn't exist.
        if !path.starts_with(&self.workdir) {
            Err(LuaError::runtime("Path may not exit working directory!"))?
        }

        if path.is_file() {
            self.assets.push((filepath, newpath)); // Will validate new path later during build.
            Ok(())
        } else {
            Err(LuaError::runtime("Invalid file!"))?
        }
    }

    fn define(&mut self, define: String) -> mlua::Result<i32> {
        self.defines.push(define);
        Ok(0)
    }
}

use std::fs;

use std::io::{BufReader, Write};

use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::null_mut;

use mlua::Lua;
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
    pub lang: String,
    pub files: Vec<PathBuf>,
    pub defines: Vec<String>,
    pub assets: Vec<(String, String)>,
    pub workspace: PathBuf,

    pub target: Option<String>,
    pub configuration: Option<String>,
    pub arch: Option<String>,
    pub toolset_compiler: Option<String>,
    pub toolset_linker: Option<String>,
    pub file: PathBuf,
    pub output: String,
    pub workdir: PathBuf,
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
            workspace: dunce::canonicalize(&args.workdir).unwrap().join(".numake"),
            target: args.target.clone(),
            configuration: args.configuration.clone(),
            arch: args.arch.clone(),
            toolset_compiler: args.toolset_compiler.clone(),
            toolset_linker: args.toolset_linker.clone(),
            file: PathBuf::from(&args.file),
            output: args.output.clone(),
            workdir: dunce::canonicalize(&args.workdir).unwrap(),
            msvc: args.msvc,
        }
    }

    pub fn setup_lua_vals(&mut self) {
        unsafe {
            PTR = self;
        }

        self.lua_instance
            .globals()
            .set("msvc", self.msvc.clone())
            .unwrap();
        self.lua_instance
            .globals()
            .set("target", self.target.clone())
            .unwrap();
        self.lua_instance
            .globals()
            .set("configuration", self.configuration.clone())
            .unwrap();
        self.lua_instance
            .globals()
            .set("arch", self.arch.clone())
            .unwrap();
        self.lua_instance
            .globals()
            .set("toolset_compiler", self.toolset_compiler.clone())
            .unwrap();
        self.lua_instance
            .globals()
            .set("toolset_linker", self.toolset_linker.clone())
            .unwrap();
        self.lua_instance
            .globals()
            .set("output", self.output.clone())
            .unwrap();

        // Functions
        self.lua_instance
            .globals()
            .set(
                "add_include_path",
                self.lua_instance
                    .create_function_mut(|_, path: String| unsafe {
                        (*PTR).add_include_path(path.clone())
                    })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "set_include_paths",
                self.lua_instance
                    .create_function_mut(|_, paths: Vec<String>| unsafe {
                        (*PTR).set_include_paths(paths)
                    })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "add_compiler_flag",
                self.lua_instance
                    .create_function_mut(|_, flag: String| unsafe {
                        (*PTR).add_compiler_flag(flag)
                    })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "set_compiler_flags",
                self.lua_instance
                    .create_function_mut(|_, flags: Vec<String>| unsafe {
                        (*PTR).set_compiler_flags(flags)
                    })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "add_linker_flag",
                self.lua_instance
                    .create_function_mut(|_, flag: String| unsafe { (*PTR).add_linker_flag(flag) })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "set_linker_flags",
                self.lua_instance
                    .create_function_mut(|_, flags: Vec<String>| unsafe {
                        (*PTR).set_linker_flags(flags)
                    })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "add_lib_path",
                self.lua_instance
                    .create_function_mut(|_, path: String| unsafe { (*PTR).add_lib_path(path) })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "set_lib_paths",
                self.lua_instance
                    .create_function_mut(|_, paths: Vec<String>| unsafe {
                        (*PTR).set_lib_paths(paths)
                    })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "add_lib",
                self.lua_instance
                    .create_function_mut(|_, lib: String| unsafe { (*PTR).add_lib(lib) })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "set_libs",
                self.lua_instance
                    .create_function_mut(|_, libs: Vec<String>| unsafe { (*PTR).set_libs(libs) })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "add_dir",
                self.lua_instance
                    .create_function_mut(|_, (path, recursive): (String, bool)| unsafe {
                        (*PTR).add_dir(
                            &Path::new(&(*PTR).workdir).parent().unwrap().join(path),
                            recursive,
                        )
                    })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "add_file",
                self.lua_instance
                    .create_function_mut(|_, path: String| unsafe {
                        (*PTR).add_file(&Path::new(&(*PTR).workdir).join(path))
                    })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "add_asset",
                self.lua_instance
                    .create_function_mut(|_, (filepath, newpath): (String, String)| unsafe {
                        (*PTR).add_asset(filepath, newpath)
                    })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "define",
                self.lua_instance
                    .create_function_mut(|_, define: String| unsafe { (*PTR).define(define) })
                    .unwrap(),
            )
            .unwrap();

        self.lua_instance
            .globals()
            .set(
                "workspace_download_zip",
                self.lua_instance
                    .create_function_mut(|_, url: String| unsafe {
                        (*PTR).workspace_download_zip(url)
                    })
                    .unwrap(),
            )
            .unwrap();

        // Get and require an online numake script, for package management mainly.
        self.lua_instance
            .globals()
            .set(
                "require_url",
                self.lua_instance
                    .create_function_mut(|_, url: String| unsafe { (*PTR).require_url(url) })
                    .unwrap(),
            )
            .unwrap();
    }

    pub fn process(&mut self) {
        if !self.workspace.exists() {
            fs::create_dir_all(&self.workspace).unwrap();
        }

        // Parse file
        self.lua_instance
            .load(fs::read_to_string(Path::new(&self.workdir).join(&self.file)).unwrap())
            .exec()
            .unwrap();

        self.toolset_compiler = self.lua_instance.globals().get("toolset_compiler").ok();
        self.toolset_linker = self.lua_instance.globals().get("toolset_linker").ok();
        self.target = self.lua_instance.globals().get("target").unwrap();
        self.arch = self.lua_instance.globals().get("arch").unwrap();
        self.configuration = self.lua_instance.globals().get("configuration").unwrap();

        self.output = self.lua_instance.globals().get("output").unwrap();
        self.msvc = self.lua_instance.globals().get("msvc").unwrap();
    }

    pub fn build(&mut self) -> Option<()> {
        if self.toolset_compiler == None {
            Err("ERROR: NO COMPILER SPECIFIED!").unwrap()
        }

        if self.toolset_linker == None {
            Err("ERROR: NO LINKER SPECIFIED!").unwrap()
        }

        let config: String = format!(
            "{}-{}-{}",
            self.arch.clone()?,
            self.target.clone()?,
            self.configuration.clone()?
        );

        let obj_dir: PathBuf = self.workspace.join(format!("obj/{}", &config));
        let out_dir: PathBuf = self.workspace.join(format!("out/{}", &config));

        if !obj_dir.exists() {
            fs::create_dir_all(&obj_dir).unwrap()
        }

        if !out_dir.exists() {
            fs::create_dir_all(&out_dir).unwrap()
        }

        let mut o_files: Vec<String> = Vec::new(); // Can't assume all compilers support wildcards.

        for file in self.files.clone() {
            let mut compiler = Command::new(&self.toolset_compiler.clone()?);

            let o_file = format!(
                "{}/{}.{}",
                &obj_dir.to_str().unwrap(),
                &file.file_name().unwrap().to_str().unwrap(),
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

            compiler_args.push(file.to_str().unwrap().to_string());

            println!(
                "{} exited with {}.",
                self.toolset_compiler.clone().unwrap(),
                compiler
                    .args(&compiler_args)
                    .current_dir(dunce::canonicalize(&self.workdir).unwrap())
                    .status()
                    .unwrap()
            );
        }

        let mut linker = Command::new(&self.toolset_linker.clone()?);
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
                &out_dir.to_str().unwrap(),
                &self.output
            ));
        } else {
            linker_args.push(format!("-o{}/{}", &out_dir.to_str().unwrap(), &self.output));
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
                .current_dir(dunce::canonicalize(&self.workdir).unwrap())
                .status()
                .unwrap()
        );

        for (oldpath, newpath) in self.assets.clone() {
            let old_path = Path::new(&oldpath); // Already canonicalized in project.rs
            let new_path = Path::new(&out_dir).join(Path::new(&newpath));

            if old_path.starts_with(dunce::canonicalize(&self.workdir).unwrap()) {
                if new_path.starts_with(&out_dir) {
                    // Make sure we haven't escaped our output dir
                    fs::copy(old_path, new_path).unwrap();
                } else {
                    Err(format!(
                        "Asset file '{}' copied to invalid destination! ({})",
                        old_path.to_str().unwrap(),
                        new_path.to_str().unwrap()
                    ))
                    .unwrap()
                }
            } else {
                Err(format!(
                    "Asset file '{}' originates outside workspace!",
                    old_path.to_str().unwrap()
                ))
                .unwrap()
            }
        }

        Some(())
    }

    fn workspace_download_zip(&mut self, url: String) -> mlua::Result<String> {
        let response = reqwest::blocking::get(&url).unwrap();

        if response.status().is_success() {
            // Where the archive will be extracted.
            let buf: [u8; 16] = *url.as_bytes().last_chunk::<16>().unwrap();
            let path = format!(
                "{}/remote/{}",
                self.workspace.to_str().unwrap(),
                Uuid::new_v8(buf)
            );
            if fs::metadata(&path).is_err() {
                // Don't "download" again. (data already in memory)
                fs::create_dir_all(&path).unwrap();
                let path_buf = dunce::canonicalize(&path).unwrap();
                let mut tempfile = tempfile()?; // Create a tempfile as a buffer for our response bytes because nothing else implements Seek ffs
                tempfile.write_all(response.bytes().unwrap().as_ref())?;

                let mut zip = ZipArchive::new(BufReader::new(tempfile)).unwrap();
                zip.extract(&path_buf).unwrap();
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

    fn add_dir(&mut self, pathbuf: &PathBuf, recursive: bool) -> mlua::Result<i32> {
        for entry in fs::read_dir(pathbuf)? {
            let path = dunce::canonicalize(entry?.path()).unwrap();
            if path.is_dir() && recursive {
                self.add_dir(&path, true)?;
            }
            if path.is_file() {
                if path.extension().unwrap() == "cpp"
                    || path.extension().unwrap() == "c"
                    || path.extension().unwrap() == "cxx"
                    || path.extension().unwrap() == "h"
                    || path.extension().unwrap() == "hxx"
                    || path.extension().unwrap() == "hpp"
                {
                    self.files.push(path);
                }
            }
        }
        Ok(0)
    }

    fn add_file(&mut self, file: &PathBuf) -> mlua::Result<i32> {
        let path = dunce::canonicalize(file).unwrap();
        if path.is_file()
            && (path.extension().unwrap() == "cpp"
                || path.extension().unwrap() == "c"
                || path.extension().unwrap() == "cxx"
                || path.extension().unwrap() == "h"
                || path.extension().unwrap() == "hxx"
                || path.extension().unwrap() == "hpp")
        {
            self.files.push(path);
            Ok(0)
        } else {
            Err("Invalid file!").unwrap()
        }
    }

    fn add_asset(&mut self, filepath: String, newpath: String) -> mlua::Result<i32> {
        let path = dunce::canonicalize(filepath.to_string()).unwrap(); // Will automatically error if path doesn't exist WHICH IS STUPID AS HELL. Don't need to join to workdir since we already assume it is relative to it. Will validate in main.
        if path.is_file() {
            self.assets.push((filepath, newpath)); // Will validate new path later in main
            Ok(0)
        } else {
            Err("Invalid file!").unwrap()
        }
    }

    fn define(&mut self, define: String) -> mlua::Result<i32> {
        self.defines.push(define);
        Ok(0)
    }
}

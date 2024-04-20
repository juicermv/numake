use std::fs;
use std::path::{PathBuf};

pub struct Project {
    pub compiler_flags: Vec<String>,
    pub linker_flags: Vec<String>,
    pub include_paths: Vec<String>,
    pub lib_paths: Vec<String>,
    pub libs: Vec<String>,
    pub lang: String,
    pub files: Vec<PathBuf>,
    pub defines: Vec<String>,
    pub assets: Vec<(String, String)>,
    pub workdir: String
}

pub static mut PROJECT: Project = Project::new();
impl Project {
    pub const fn new() -> Self {
        Project {
            compiler_flags: Vec::new(),
            linker_flags: Vec::new(),
            include_paths: Vec::new(),
            lib_paths: Vec::new(),
            libs: Vec::new(),
            lang: String::new(),
            files: Vec::new(),
            defines: Vec::new(),
            assets: Vec::new(),
            workdir: String::new()
        }
    }

    pub fn add_include_path(&mut self, path:String) -> mlua::Result<i32> {
        self.include_paths.push(path);
        Ok(0)
    }

    pub fn set_include_paths(&mut self, paths:Vec<String>) -> mlua::Result<i32>{
        self.include_paths = paths.to_vec();
        Ok(0)
    }

    pub fn add_compiler_flag(&mut self, flag:String) -> mlua::Result<i32> {
        self.compiler_flags.push(flag);
        Ok(0)
    }

    pub fn set_compiler_flags(&mut self,  flags:Vec<String>) -> mlua::Result<i32>{
        self.compiler_flags = flags;
        Ok(0)
    }

    pub fn add_linker_flag(&mut self, flag:String) -> mlua::Result<i32> {
        self.linker_flags.push(flag);
        Ok(0)
    }

    pub fn set_linker_flags(&mut self,  flags: Vec<String>) -> mlua::Result<i32>{
        self.linker_flags = flags;
        Ok(0)
    }

    pub fn add_lib_path(&mut self, path:String) -> mlua::Result<i32>{
        self.lib_paths.push(path);
        Ok(0)
    }

    pub fn set_lib_paths(&mut self, paths:Vec<String>) -> mlua::Result<i32>{
        self.lib_paths = paths.to_vec();
        Ok(0)
    }

    pub fn add_lib(&mut self, lib:String) -> mlua::Result<i32>{
        self.libs.push(lib);
        Ok(0)
    }

    pub fn set_libs(&mut self, libs:Vec<String>) -> mlua::Result<i32>{
        self.libs = libs;
        Ok(0)
    }

    pub fn add_dir(&mut self, pathbuf: &PathBuf, recursive: bool) -> mlua::Result<i32>{
        for entry in fs::read_dir(pathbuf)? {
            let path = dunce::canonicalize(entry?.path()).unwrap();
            if path.is_dir() && recursive {
                self.add_dir(&path, true)?;
            }
            if  path.is_file() {
                if
                    path.extension().unwrap() == "cpp"
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

    pub fn add_file(&mut self, file: &PathBuf) -> mlua::Result<i32>{
        let path = dunce::canonicalize(file).unwrap();
        if path.is_file() &&
            (
                path.extension().unwrap() == "cpp"
                || path.extension().unwrap() == "c"
                || path.extension().unwrap() == "cxx"
                || path.extension().unwrap() == "h"
                || path.extension().unwrap() == "hxx"
                || path.extension().unwrap() == "hpp"
            )

        {
            self.files.push(path);
            Ok(0)
        } else {
            Err("Invalid file!").unwrap()
        }
    }

    pub fn add_asset(&mut self, filepath:String, newpath: String) -> mlua::Result<i32>{
        let path = dunce::canonicalize(filepath.to_string()).unwrap(); // Will automatically error if path doesn't exist WHICH IS STUPID AS HELL. Don't need to join to workdir since we already assume it is relative to it. Will validate in main.
        if path.is_file() {
            self.assets.push((filepath, newpath)); // Will validate new path later in main
            Ok(0)
        } else {
            Err("Invalid file!").unwrap()
        }
    }

    pub fn define(&mut self, define: String) -> mlua::Result<i32>{
        self.defines.push(define);
        Ok(0)
    }
}
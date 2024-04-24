use std::collections::HashMap;
use std::fs;

use crate::error::{to_lua_result, NUMAKE_ERROR};
use mlua::prelude::LuaError;
use mlua::{FromLua, Lua, Table, UserData, UserDataFields, UserDataMethods, Value};
use std::path::PathBuf;

#[derive(Clone)]
pub struct Target {
    pub compiler_flags: Vec<String>,
    pub linker_flags: Vec<String>,
    pub include_paths: Vec<String>,
    pub lib_paths: Vec<String>,
    pub libs: Vec<String>,
    pub defines: Vec<String>,

    pub assets: HashMap<String, String>,

    pub lang: String,
    pub output: Option<String>,

    pub files: Vec<PathBuf>,

    pub toolset_compiler: Option<String>,
    pub toolset_linker: Option<String>,
    pub msvc: bool,

    pub name: String,

    workdir: PathBuf,
}

impl Target {
    pub fn new(
        name: String,
        toolset_compiler: Option<String>,
        toolset_linker: Option<String>,
        msvc: bool,
        output: Option<String>,
        workdir: PathBuf,
    ) -> anyhow::Result<Self> {
        Ok(Target {
            compiler_flags: Vec::new(),
            linker_flags: Vec::new(),
            include_paths: Vec::new(),
            lib_paths: Vec::new(),
            libs: Vec::new(),
            lang: String::new(),
            output,
            files: Vec::new(),
            toolset_compiler,
            toolset_linker,
            defines: Vec::new(),
            assets: HashMap::new(),
            msvc,
            workdir,
            name,
        })
    }

    pub fn add_file(&mut self, file: PathBuf) -> anyhow::Result<()> {
        if !file.starts_with(&self.workdir) {
            Err(mlua::Error::runtime(&NUMAKE_ERROR.PATH_OUTSIDE_WORKING_DIR))?
        }

        if file.is_file() {
            self.files.push(file.clone());
        } else {
            Err(mlua::Error::runtime(&NUMAKE_ERROR.ADD_FILE_IS_DIRECTORY))?
        }
        Ok(())
    }

    pub fn add_dir(&mut self, path_buf: PathBuf, recursive: bool) -> anyhow::Result<()> {
        if !path_buf.starts_with(&self.workdir) {
            Err(LuaError::runtime(&NUMAKE_ERROR.PATH_OUTSIDE_WORKING_DIR))?
        }

        for entry in fs::read_dir(path_buf)? {
            let path = dunce::canonicalize(entry?.path())?;
            if path.is_dir() && recursive {
                self.add_dir(path.clone(), true)?
            }
            if path.is_file() {
                self.add_file(path.clone())?
            }
        }
        Ok(())
    }
}
impl UserData for Target {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        {
            fields.add_field_method_get("include_paths", |_, this| Ok(this.include_paths.clone()));

            fields.add_field_method_set("include_paths", |_, this, val: Vec<String>| {
                Ok(this.include_paths = val)
            });
        }

        {
            fields.add_field_method_get("library_paths", |_, this| Ok(this.lib_paths.clone()));

            fields.add_field_method_set("library_paths", |_, this, val: Vec<String>| {
                Ok(this.lib_paths = val)
            });
        }

        {
            fields.add_field_method_get("libraries", |_, this| Ok(this.libs.clone()));

            fields
                .add_field_method_set("libraries", |_, this, val: Vec<String>| Ok(this.libs = val));
        }

        {
            fields.add_field_method_get("definitions", |_, this| Ok(this.defines.clone()));

            fields.add_field_method_set("definitions", |_, this, val: Vec<String>| {
                Ok(this.defines = val)
            });
        }

        {
            fields.add_field_method_get("compiler", |_, this| Ok(this.toolset_compiler.clone()));

            fields.add_field_method_set("compiler", |_, this, val: Option<String>| {
                Ok(this.toolset_compiler = val)
            });
        }

        {
            fields.add_field_method_get("linker", |_, this| Ok(this.toolset_linker.clone()));

            fields.add_field_method_set("linker", |_, this, val: Option<String>| {
                Ok(this.toolset_linker = val)
            });
        }

        {
            fields.add_field_method_get("output", |_, this| Ok(this.output.clone()));

            fields.add_field_method_set("output", |_, this, val: Option<String>| {
                Ok(this.output = val)
            });
        }

        {
            fields.add_field_method_get("use_msvc", |_, this| Ok(this.msvc.clone()));

            fields.add_field_method_set("use_msvc", |_, this, val: bool| Ok(this.msvc = val));
        }

        {
            fields
                .add_field_method_get("compiler_flags", |_, this| Ok(this.compiler_flags.clone()));

            fields.add_field_method_set("compiler_flags", |_, this, val: Vec<String>| {
                Ok(this.compiler_flags = val)
            });
        }

        {
            fields.add_field_method_get("linker_flags", |_, this| Ok(this.linker_flags.clone()));

            fields.add_field_method_set("linker_flags", |_, this, val: Vec<String>| {
                Ok(this.linker_flags = val)
            });
        }

        {
            fields.add_field_method_get("files", |_, this| {
                let return_val: Vec<String> = this
                    .files
                    .clone()
                    .into_iter()
                    .map(|value| {
                        return value
                            .to_str()
                            .unwrap_or("ERROR")
                            .to_string()
                            .replace(this.workdir.to_str().unwrap_or("ERROR"), "");
                    })
                    .collect();
                Ok(return_val)
            });

            fields.add_field_method_set("files", |_, this, val: Vec<String>| {
                for path in val {
                    to_lua_result(this.add_file(dunce::canonicalize(this.workdir.join(path))?))?
                }

                Ok(())
            });
        }

        {
            fields.add_field_method_get("assets", |_, this| Ok(this.assets.clone()));

            fields.add_field_method_set("assets", |_, this, val: Table| {
                val.for_each::<String, String>(|old_path, new_path| {
                    let path = &dunce::canonicalize(this.workdir.join(&old_path))?; // Will automatically error if path doesn't exist.
                    if !path.starts_with(&this.workdir) {
                        Err(mlua::Error::runtime(&NUMAKE_ERROR.PATH_OUTSIDE_WORKING_DIR))?
                    }

                    this.assets.insert(old_path, new_path); // Will validate new path later during build.
                    Ok(())
                })
            });
        }
    }
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("add_dir", |_, this, (path, recursive): (String, bool)| {
            to_lua_result(this.add_dir(dunce::canonicalize(this.workdir.join(path))?, recursive))
        });
    }
}

impl<'lua> FromLua<'lua> for Target {
    fn from_lua(value: Value<'lua>, _: &'lua Lua) -> mlua::Result<Self> {
        match value {
            Value::UserData(user_data) => Ok(user_data.borrow::<Self>()?.clone()),
            _ => unreachable!(),
        }
    }
}

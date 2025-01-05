use crate::lib::data::project_type::ProjectType;
use crate::lib::data::source_file_collection::SourceFileCollection;
use crate::lib::util::error::NuMakeError::{
	AddFileIsDirectory, AssetCopyPathOutsideWorkingDirectory,
};
use anyhow::anyhow;
use mlua::{FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use mlua::prelude::LuaValue;
use crate::lib::data::project_language::ProjectLanguage;

#[derive(Debug, Clone, Default)]
pub struct Project {
	pub name: String,
	pub language: ProjectLanguage,

	pub output: Option<String>,

	pub source_files: SourceFileCollection,
	pub asset_files: HashMap<String, String>,

	pub include_paths: Vec<String>,
	pub lib_paths: Vec<String>,

	pub libs: Vec<String>,
	pub defines: Vec<String>,

	pub compiler_flags: Vec<String>,
	pub linker_flags: Vec<String>,
	pub rc_flags: Vec<String>,
	pub windres_flags: Vec<String>,

	pub arch: Option<String>,
	pub project_type: ProjectType,
}

impl Project {
	pub fn add_file(
		&mut self,
		file: &PathBuf,
	) -> anyhow::Result<()> {
		if file.is_file() {
			self.source_files.insert(file)
		} else {
			return Err(anyhow!(AddFileIsDirectory));
		}
		Ok(())
	}

	pub fn copy_assets(
		&self,
		working_directory: &PathBuf,
		out_dir: &PathBuf,
	) -> anyhow::Result<()> {
		for (key, val) in &self.asset_files {
			let original_path =
				dunce::canonicalize(working_directory.join(Path::new(&key)))?;
			let copy_path = out_dir.join(Path::new(&val));
			if !copy_path.starts_with(out_dir) {
				Err(anyhow!(AssetCopyPathOutsideWorkingDirectory))?
			} else {
				fs::copy(original_path, copy_path)?;
			}
		}

		Ok(())
	}
}

impl UserData for Project {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("name", |_, this| Ok(this.name.clone()));
		fields.add_field_method_get("projectLanguage", |_, this| Ok(this.language.clone()));
		fields.add_field_method_get("output", |_, this| {
			Ok(this.output.clone().unwrap_or("out".into()))
		});
		fields.add_field_method_get("assetFiles", |_, this| {
			Ok(this.asset_files.clone())
		});
		fields.add_field_method_get("includePaths", |_, this| {
			Ok(this.include_paths.clone())
		});
		fields.add_field_method_get("libPaths", |_, this| Ok(this.libs.clone()));
		fields.add_field_method_get("libs", |_, this| Ok(this.libs.clone()));
		fields.add_field_method_get("defines", |_, this| {
			Ok(this.defines.clone())
		});
		fields.add_field_method_get("compilerFlags", |_, this| {
			Ok(this.compiler_flags.clone())
		});
		fields.add_field_method_get("linkerFlags", |_, this| {
			Ok(this.linker_flags.clone())
		});
		fields.add_field_method_get("rcFlags", |_, this| {
			Ok(this.rc_flags.clone())
		});
		fields.add_field_method_get("windresFlags", |_, this| {
			Ok(this.windres_flags.clone())
		});
		fields.add_field_method_get("arch", |_, this| Ok(this.arch.clone()));
		fields.add_field_method_get("project_type", |_, this| {
			Ok(this.project_type.clone())
		});


		// SETTERS
		fields.add_field_method_set("name", |_, this, new_val| {
			this.name = new_val;
			Ok(())
		});

		fields.add_field_method_set("projectLanguage", |_, this, new_val| {
			this.language = new_val;
			Ok(())
		});

		fields.add_field_method_set("output", |_, this, new_val| {
			this.output = new_val;
			Ok(())
		});

		fields.add_field_method_set(
			"sourceFiles",
			|_, this, new_val: Vec<String>| {
				for path in new_val {
					this.source_files.insert(dunce::canonicalize(Path::new(&path))?);
				}
				Ok(())
			},
		);

		fields.add_field_method_set("assetFiles", |_, this, new_val: HashMap<String, String>| {
			this.asset_files = new_val;
            Ok(())
		});

		fields.add_field_method_set("includePaths", |_, this, new_val: Vec<String>| {
			this.include_paths = new_val.clone();
			Ok(())
		});

		fields.add_field_method_set("libPaths", |_, this, new_val| {
			this.lib_paths = new_val;
			Ok(())
		});

		fields.add_field_method_set("libs", |_, this, new_val| {
			this.libs = new_val;
			Ok(())
		});

		fields.add_field_method_set("defines", |_, this, new_val| {
			this.defines = new_val;
			Ok(())
		});

		fields.add_field_method_set("compilerFlags", |_, this, new_val| {
			this.compiler_flags = new_val;
			Ok(())
		});

		fields.add_field_method_set("linkerFlags", |_, this, new_val| {
			this.linker_flags = new_val;
			Ok(())
		});

		fields.add_field_method_set("rcFlags", |_, this, new_val| {
			this.rc_flags = new_val;
			Ok(())
		});

		fields.add_field_method_set("windresFlags", |_, this, new_val| {
			this.rc_flags = new_val;
			Ok(())
		});

		fields.add_field_method_set("arch", |_, this, new_val| {
			this.arch = new_val;
			Ok(())
		});

		fields.add_field_method_set("projectType", |_, this, new_val| {
			this.project_type = new_val;
			Ok(())
		});
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_meta_method(
			MetaMethod::Call,
			|lua,
			 this,
			 (
				name,
			 	language,
				output,
				source_files,
				asset_files,
				include_paths,
				lib_paths,
				libs,
				defines,
				compiler_flags,
				linker_flags,
				rc_flags,
			 	windres_flags,
				arch,
				project_type,
			): (
				String,
				ProjectLanguage,
				String,
				Vec<String>,
				HashMap<String, String>,
				Vec<String>,
				Vec<String>,
				Vec<String>,
				Vec<String>,
				Vec<String>,
				Vec<String>,
				Vec<String>,
				Vec<String>,
				String,
				ProjectType,
			)| {
                let mut new_proj = Project {
                    name,
					language,
                    output: Some(output),
                    source_files: SourceFileCollection::new(),
                    asset_files,
                    include_paths,
                    lib_paths,
                    libs,
                    defines,
                    compiler_flags,
                    linker_flags,
                    rc_flags,
					windres_flags,
                    arch: Some(arch),
                    project_type
                };

                for file in source_files {
                    new_proj.source_files.insert(dunce::canonicalize(Path::new(&file))?);
                }

                Ok(new_proj)
            },
		)
	}
}

impl FromLua for Project {
	fn from_lua(
		value: LuaValue,
		_: &Lua,
	) -> mlua::Result<Self> {
		match value {
			Value::UserData(user_data) => {
				if user_data.is::<Self>() {
					Ok(user_data.borrow::<Self>()?.clone())
				} else {
					Err(mlua::Error::UserDataTypeMismatch)
				}
			}

			_ => Err(mlua::Error::UserDataTypeMismatch),
		}
	}
}


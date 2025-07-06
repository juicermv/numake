use crate::lib::data::project_type::ProjectType;
use crate::lib::data::source_file_collection::SourceFileCollection;
use crate::lib::util::error::NuMakeError::{
	AddFileIsDirectory, AssetCopyPathOutsideWorkingDirectory,
};
use anyhow::anyhow;
use mlua::{FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
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
				dunce::canonicalize(working_directory.join(key))?;
			let copy_path = out_dir.join(val);
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
		fields.add_field_method_get("project_language", |_, this| Ok(this.language.clone()));
		fields.add_field_method_get("output", |_, this| {
			Ok(this.output.clone().unwrap_or("out".into()))
		});
		fields.add_field_method_get("asset_files", |_, this| {
			Ok(this.asset_files.clone())
		});
		fields.add_field_method_get("include_paths", |_, this| {
			Ok(this.include_paths.clone())
		});
		fields.add_field_method_get("lib_paths", |_, this| Ok(this.libs.clone()));
		fields.add_field_method_get("libs", |_, this| Ok(this.libs.clone()));
		fields.add_field_method_get("defines", |_, this| {
			Ok(this.defines.clone())
		});
		fields.add_field_method_get("compiler_flags", |_, this| {
			Ok(this.compiler_flags.clone())
		});
		fields.add_field_method_get("linker_flags", |_, this| {
			Ok(this.linker_flags.clone())
		});
		fields.add_field_method_get("rc_flags", |_, this| {
			Ok(this.rc_flags.clone())
		});
		fields.add_field_method_get("windres_flags", |_, this| {
			Ok(this.windres_flags.clone())
		});
		fields.add_field_method_get("arch", |_, this| Ok(this.arch.clone()));
		fields.add_field_method_get("project_type", |_, this| {
			Ok(this.project_type.clone())
		});


		// SETTERS ---------------------------------------------------------------------------------
		fields.add_field_method_set("name", |_, this, new_val| {
			this.name = new_val;
			Ok(())
		});

		fields.add_field_method_set("project_language", |_, this, new_val| {
			this.language = new_val;
			Ok(())
		});fields.add_field_method_set("output", |_, this, new_val| {
			this.output = new_val;
			Ok(())
		});

		fields.add_field_method_set(
			"source_files",
			|_, this, new_val: Vec<String>| {
				this.source_files.clear();
				for path in new_val {
					this.source_files.insert(dunce::canonicalize(&path)?);
				}
				Ok(())
			},
		);

		fields.add_field_method_set("asset_files", |_, this, new_val: HashMap<String, String>| {
			this.asset_files = new_val;
            Ok(())
		});

		fields.add_field_method_set("include_paths", |_, this, new_val: Vec<String>| {
			this.include_paths = new_val.clone();
			Ok(())
		});

		fields.add_field_method_set("lib_paths", |_, this, new_val| {
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

		fields.add_field_method_set("compiler_flags", |_, this, new_val| {
			this.compiler_flags = new_val;
			Ok(())
		});

		fields.add_field_method_set("linker_flags", |_, this, new_val| {
			this.linker_flags = new_val;
			Ok(())
		});

		fields.add_field_method_set("rc_flags", |_, this, new_val| {
			this.rc_flags = new_val;
			Ok(())
		});

		fields.add_field_method_set("windres_flags", |_, this, new_val| {
			this.rc_flags = new_val;
			Ok(())
		});

		fields.add_field_method_set("arch", |_, this, new_val| {
			this.arch = new_val;
			Ok(())
		});

		fields.add_field_method_set("project_type", |_, this, new_val| {
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
				Option<String>,
				Option<ProjectLanguage>,
				Option<String>,
				Option<Vec<String>>,
				Option<HashMap<String, String>>,
				Option<Vec<String>>,
				Option<Vec<String>>,
				Option<Vec<String>>,
				Option<Vec<String>>,
				Option<Vec<String>>,
				Option<Vec<String>>,
				Option<Vec<String>>,
				Option<Vec<String>>,
				Option<String>,
				Option<ProjectType>,
			)| {
                let mut new_proj = Project {
                    name: name.unwrap_or_default(),
					language: language.unwrap_or_default(),
                    output,
                    source_files: SourceFileCollection::new(),
                    asset_files: asset_files.unwrap_or_default(),
                    include_paths: include_paths.unwrap_or_default(),
                    lib_paths: lib_paths.unwrap_or_default(),
                    libs: libs.unwrap_or_default(),
                    defines: defines.unwrap_or_default(),
                    compiler_flags: compiler_flags.unwrap_or_default(),
                    linker_flags: linker_flags.unwrap_or_default(),
                    rc_flags: rc_flags.unwrap_or_default(),
					windres_flags: windres_flags.unwrap_or_default(),
                    arch,
                    project_type: project_type.unwrap_or_default(),
                };

                for file in source_files.unwrap_or_default() {
                    new_proj.source_files.insert(dunce::canonicalize(&file)?);
                }

                Ok(new_proj)
            },
		);
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


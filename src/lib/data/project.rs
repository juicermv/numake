use crate::lib::compilers::msvc::MSVC;
use crate::lib::data::flag_type::FlagType;
use crate::lib::data::project_language::ProjectLanguage;
use crate::lib::data::project_type::ProjectType;
use crate::lib::data::source_file_collection::SourceFileCollection;
use crate::lib::util::either::Either;
use crate::lib::util::error::NuMakeError::{
	AddFileIsDirectory, AssetCopyPathOutsideWorkingDirectory,
};
use anyhow::anyhow;
use mlua::prelude::LuaValue;
use mlua::{
	FromLua, Lua, MetaMethod, UserData, UserDataFields, UserDataMethods, Value,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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

	pub flags: Vec<(FlagType, String)>,

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

	pub fn get_flags(
		&self,
		flag_type: FlagType,
	) -> Vec<String> {
		self.flags
			.iter()
			.filter_map(|(ft, flag)| {
				if ft.clone() == flag_type {
					Some(flag.clone())
				} else {
					None
				}
			})
			.collect()
	}
}

impl UserData for Project {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut(
			"file",
			|_, this, value: Either<String, Vec<String>>| {
				match value {
					Either::First(path) => {
						let path_buf = dunce::canonicalize(path)?;
						this.source_files.insert(path_buf);
					}

					Either::Second(paths) => {
						for path in paths {
							let path_buf = dunce::canonicalize(path)?;
							this.source_files.insert(path_buf);
						}
					}
				}

				Ok(())
			},
		);

		methods.add_method_mut("output", |_, this, path: String| {
			this.output = Some(path);
			Ok(())
		});

		methods.add_method_mut(
			"asset",
			|_, this, (path, out): (String, String)| {
				this.asset_files.insert(path, out);
				Ok(())
			},
		);
    
		methods.add_method_mut("include", |_, this, value: Either<String, Vec<String>>| {
			match value {
				Either::First(path) => this.include_paths.push(path),
				Either::Second(paths) => this.include_paths.extend(paths),
			}
			Ok(())
		});

		methods.add_method_mut(
			"lib",
			|_, this, value: Either<String, Vec<String>>| match value {
				Either::First(str) => {
					this.libs.push(str);
					Ok(())
				}

				Either::Second(arr) => {
					this.libs.extend(arr);
					Ok(())
				}
			},
		);

		methods.add_method_mut("lib_path", |_, this, value: Either<String, Vec<String>>| {
			match value {
				Either::First(path) => this.lib_paths.push(path),
				Either::Second(paths) => this.lib_paths.extend(paths),
			}
			Ok(())
		});

		methods.add_method_mut(
			"define",
			|_, this, value: Either<String, Vec<String>>| {
				match value {
					Either::First(str) => this.defines.push(str),
					Either::Second(arr) => this.defines.extend(arr)
				}
				Ok(())
			},
		);

		methods.add_method_mut(
			"flag",
			|_,
			 this,
			 (flag_type, value): (FlagType, Either<String, Vec<String>>)| {
				match value {
					Either::First(str) => {
						this.flags.push((flag_type, str));
						Ok(())
					}

					Either::Second(arr) => {
						this.flags.extend(
							arr.iter()
								.map(|str| (flag_type.clone(), str.clone()))
								.collect::<Vec<(FlagType, String)>>(),
						);
						Ok(())
					}
				}
			},
		);

		methods.add_method_mut("arch", |_, this, arch: String| {
			this.arch = Some(arch);
			Ok(())
		});

		methods.add_method_mut("type", |_, this, project_type: ProjectType| {
			this.project_type = project_type;
			Ok(())
		});
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

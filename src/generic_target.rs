use std::{
	collections::HashMap,
	fs

	,
	path::PathBuf,
	process::{
		Command,
		Stdio,
	},
};

use anyhow::anyhow;
use mlua::{
	FromLua,
	Lua,
	Table,
	UserData,
	UserDataFields,
	UserDataMethods,
	Value,
};
use pathdiff::diff_paths;
use serde::Serialize;

use crate::{
	error::NUMAKE_ERROR,
	lua_workspace::LuaWorkspace,
	util::{
		log,
		to_lua_result,
	},
};
use crate::target::TargetTrait;

#[derive(Clone, Serialize)]
pub struct GenericTarget
{
	pub compiler_flags: Vec<String>,
	pub linker_flags: Vec<String>,
	pub include_paths: Vec<String>,
	pub lib_paths: Vec<String>,
	pub libs: Vec<String>,
	pub defines: Vec<String>,

	pub assets: HashMap<PathBuf, String>,
	pub output: Option<String>,

	pub files: Vec<PathBuf>,

	pub toolset_compiler: Option<String>,
	pub toolset_linker: Option<String>,

	pub name: String,

	workdir: PathBuf,

	#[serde(skip_serializing)]
	quiet: bool,
}

impl GenericTarget
{
	pub fn new(
		name: String,
		toolset_compiler: Option<String>,
		toolset_linker: Option<String>,
		output: Option<String>,
		workdir: PathBuf,
		quiet: bool,
	) -> anyhow::Result<Self>
	{
		Ok(GenericTarget {
			compiler_flags: Vec::new(),
			linker_flags: Vec::new(),
			include_paths: Vec::new(),
			lib_paths: Vec::new(),
			libs: Vec::new(),
			output,
			files: Vec::new(),
			toolset_compiler,
			toolset_linker,
			defines: Vec::new(),
			assets: HashMap::new(),
			workdir,
			quiet,
			name,
		})
	}

	pub fn add_file(
		&mut self,
		file: PathBuf,
	) -> anyhow::Result<()>
	{
		if !file.starts_with(&self.workdir) {
			Err(mlua::Error::runtime(NUMAKE_ERROR.PATH_OUTSIDE_WORKING_DIR))?
		}

		if file.is_file() {
			self.files.push(file.clone());
		} else {
			Err(mlua::Error::runtime(NUMAKE_ERROR.ADD_FILE_IS_DIRECTORY))?
		}
		Ok(())
	}

	fn copy_assets(
		&self,
		out_dir: &PathBuf,
	) -> anyhow::Result<()>
	{
		for (key, val) in self.assets.clone() {
			let copy_path = out_dir.join(val);
			if !copy_path.starts_with(out_dir) {
				Err(anyhow!(NUMAKE_ERROR.ASSET_COPY_PATH_OUTSIDE_OUTPUT_DIR))?
			} else {
				fs::copy(key, copy_path)?;
			}
		}

		Ok(())
	}
}

impl TargetTrait for GenericTarget {
	fn build(
		&self,
		parent_workspace: &mut LuaWorkspace,
	) -> anyhow::Result<()>
	{
		let obj_dir: PathBuf = parent_workspace
			.workspace
			.join(format!("obj/{}", &self.name));
		let out_dir: PathBuf = parent_workspace
			.workspace
			.join(format!("out/{}", &self.name));

		if !obj_dir.exists() {
			fs::create_dir_all(&obj_dir)?;
		}

		if !out_dir.exists() {
			fs::create_dir_all(&out_dir)?;
		}

		let mut o_files: Vec<String> = Vec::new(); // Can't assume all compilers support wildcards.

		let toolset_compiler: Option<String> =
			if parent_workspace.toolset_compiler.is_none() {
				self.toolset_compiler.clone()
			} else {
				parent_workspace.toolset_compiler.clone()
			};

		let toolset_linker: Option<String> =
			if parent_workspace.toolset_linker.is_none() {
				self.toolset_linker.clone()
			} else {
				parent_workspace.toolset_linker.clone()
			};

		let output: Option<String> = if parent_workspace.output.is_none() {
			self.output.clone()
		} else {
			parent_workspace.output.clone()
		};

		if toolset_linker.is_none() {
			Err(anyhow!(&NUMAKE_ERROR.TOOLSET_LINKER_NULL))?
		}

		if toolset_compiler.is_none() {
			Err(anyhow!(&NUMAKE_ERROR.TOOLSET_LINKER_NULL))?
		}

		for file in self.files.clone() {
			let mut compiler = Command::new(
				toolset_compiler.clone().unwrap_or("NULL".to_string()),
			);

			let o_file = obj_dir.join(
				diff_paths(&file, self.workdir.clone())
					.unwrap()
					.to_str()
					.unwrap()
					.to_string() + ".o",
			);

			if !o_file.parent().unwrap().exists() {
				fs::create_dir_all(o_file.parent().unwrap())?;
			}

			let mut compiler_args = Vec::from([
				"-c".to_string(),
				format!("-o{}", o_file.to_str().unwrap()),
			]);

			o_files.push(o_file.to_str().unwrap().to_string());

			for incl in self.include_paths.clone() {
				compiler_args.push(format!("-I{incl}"))
			}

			for define in self.defines.clone() {
				compiler_args.push(format!("-D{define}"))
			}

			for flag in self.compiler_flags.clone() {
				compiler_args.push(flag)
			}

			compiler_args.push(file.to_str().unwrap_or("ERROR").to_string());

			let status = compiler
				.stdout(
					if self.quiet {
						Stdio::null()
					} else {
						Stdio::inherit()
					},
				)
				.stderr(
					if self.quiet {
						Stdio::null()
					} else {
						Stdio::inherit()
					},
				)
				.args(&compiler_args)
				.current_dir(&parent_workspace.working_directory)
				.status()?;

			log(
				&format!(
					"\n{} exited with {}.\n",
					toolset_compiler.clone().unwrap_or("NULL".to_string()),
					status
				),
				self.quiet,
			);

			if !status.success() {
				log("Aborting...", self.quiet);
				Err(anyhow!(status))?
			}
		}

		let mut linker =
			Command::new(toolset_linker.clone().unwrap_or("NULL".to_string()));
		let mut linker_args = Vec::new();

		linker_args.append(&mut o_files);

		for path in self.lib_paths.clone() {
			linker_args.push(format!("-L{path}"))
		}

		for lib in self.libs.clone() {
			linker_args.push(format!("-l{lib}"))
		}

		linker_args.push(format!(
			"-o{}/{}",
			&out_dir.to_str().unwrap_or("ERROR"),
			&output.unwrap_or("out".to_string())
		));

		for flag in self.linker_flags.clone() {
			linker_args.push(flag)
		}

		log(
			&format!(
				"\n{} exited with {}. \n",
				toolset_linker.clone().unwrap_or("NULL".to_string()),
				linker
					.stdout(
						if self.quiet {
							Stdio::null()
						} else {
							Stdio::inherit()
						},
					)
					.stderr(
						if self.quiet {
							Stdio::null()
						} else {
							Stdio::inherit()
						},
					)
					.args(&linker_args)
					.current_dir(&parent_workspace.working_directory)
					.status()?
			),
			self.quiet,
		);

		self.copy_assets(&out_dir)?;

		Ok(())
	}
}


impl UserData for GenericTarget
{
	fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F)
	{
		{
			fields.add_field_method_get("include_paths", |_, this| {
				Ok(this.include_paths.clone())
			});

			fields.add_field_method_set(
				"include_paths",
				|_, this, val: Vec<String>| {
					this.include_paths = val;
					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("library_paths", |_, this| {
				Ok(this.lib_paths.clone())
			});

			fields.add_field_method_set(
				"library_paths",
				|_, this, val: Vec<String>| {
					this.lib_paths = val;
					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("libraries", |_, this| {
				Ok(this.libs.clone())
			});

			fields.add_field_method_set(
				"libraries",
				|_, this, val: Vec<String>| {
					this.libs = val;
					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("definitions", |_, this| {
				Ok(this.defines.clone())
			});

			fields.add_field_method_set(
				"definitions",
				|_, this, val: Vec<String>| {
					this.defines = val;
					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("compiler", |_, this| {
				Ok(this.toolset_compiler.clone())
			});

			fields.add_field_method_set(
				"compiler",
				|_, this, val: Option<String>| {
					this.toolset_compiler = val;
					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("linker", |_, this| {
				Ok(this.toolset_linker.clone())
			});

			fields.add_field_method_set(
				"linker",
				|_, this, val: Option<String>| {
					this.toolset_linker = val;
					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("output", |_, this| {
				Ok(this.output.clone())
			});

			fields.add_field_method_set(
				"output",
				|_, this, val: Option<String>| {
					this.output = val;
					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("compiler_flags", |_, this| {
				Ok(this.compiler_flags.clone())
			});

			fields.add_field_method_set(
				"compiler_flags",
				|_, this, val: Vec<String>| {
					this.compiler_flags = val;
					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("linker_flags", |_, this| {
				Ok(this.linker_flags.clone())
			});

			fields.add_field_method_set(
				"linker_flags",
				|_, this, val: Vec<String>| {
					this.linker_flags = val;
					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("files", |_, this| {
				let return_val: Vec<String> = this
					.files
					.clone()
					.into_iter()
					.map(|value| {
						return diff_paths(value, this.workdir.clone())
							.unwrap()
							.to_str()
							.unwrap()
							.to_string();
					})
					.collect();
				Ok(return_val)
			});

			fields.add_field_method_set(
				"files",
				|_, this, val: Vec<String>| {
					for path in val {
						to_lua_result(this.add_file(dunce::canonicalize(
							this.workdir.join(path),
						)?))?
					}

					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("assets", |_, this| {
				Ok(this
					.assets
					.iter()
					.map(|(key, val)| {
						(
							key.to_str().unwrap_or_default().to_string(),
							val.clone(),
						)
					})
					.collect::<HashMap<String, String>>())
			});

			fields.add_field_method_set("assets", |_, this, val: Table| {
				val.for_each(|key: String, val: String| {
					let path = dunce::canonicalize(this.workdir.join(key))?; // Will automatically error if path doesn't exist.
					if !path.starts_with(&this.workdir) {
						Err(mlua::Error::runtime(
							NUMAKE_ERROR.PATH_OUTSIDE_WORKING_DIR,
						))?
					}

					this.assets.insert(path, val); // Will validate new path later during build.

					Ok(())
				})
			});
		}
	}

	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M)
	{
		// Nothing at the moment!
	}
}

impl<'lua> FromLua<'lua> for GenericTarget
{
	fn from_lua(
		value: Value<'lua>,
		_: &'lua Lua,
	) -> mlua::Result<Self>
	{
		match value {
			Value::UserData(user_data) => {
				Ok(user_data.borrow::<Self>()?.clone())
			}
			_ => unreachable!(),
		}
	}
}
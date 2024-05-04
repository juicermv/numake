use std::{
	collections::HashMap,
	fs,
	fs::File,
	io::Write,
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
use tempfile::tempdir;

use crate::{
	error::NUMAKE_ERROR,
	lua_workspace::LuaWorkspace,
	util::{
		download_vswhere,
		log,
		to_lua_result,
	},
};
use crate::target::TargetTrait;

#[derive(Clone, Serialize)]
pub struct MSVCTarget
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

	pub name: String,

	workdir: PathBuf,

	#[serde(skip_serializing)]
	quiet: bool,

	pub resources: Vec<PathBuf>,
	pub def_files: Vec<PathBuf>,
	pub rc_flags: Vec<String>,
	arch: Option<String>,
	static_lib: bool,
}

impl MSVCTarget
{
	pub fn new(
		name: String,
		output: Option<String>,
		workdir: PathBuf,
		quiet: bool,
	) -> anyhow::Result<Self>
	{
		Ok(MSVCTarget {
			compiler_flags: Vec::new(),
			linker_flags: Vec::new(),
			include_paths: Vec::new(),
			lib_paths: Vec::new(),
			libs: Vec::new(),
			output,
			files: Vec::new(),
			defines: Vec::new(),
			assets: HashMap::new(),
			workdir,
			arch: None,
			quiet,
			resources: Vec::new(),
			def_files: Vec::new(),
			rc_flags: Vec::new(),
			static_lib: false,
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

	pub fn add_rc_file(
		&mut self,
		file: PathBuf,
	) -> anyhow::Result<()>
	{
		if !file.starts_with(&self.workdir) {
			Err(mlua::Error::runtime(NUMAKE_ERROR.PATH_OUTSIDE_WORKING_DIR))?
		}

		if file.is_file() {
			self.resources.push(file.clone());
		} else {
			Err(mlua::Error::runtime(NUMAKE_ERROR.ADD_FILE_IS_DIRECTORY))?
		}
		Ok(())
	}

	pub fn add_def_file(
		&mut self,
		file: PathBuf,
	) -> anyhow::Result<()>
	{
		if !file.starts_with(&self.workdir) {
			Err(mlua::Error::runtime(NUMAKE_ERROR.PATH_OUTSIDE_WORKING_DIR))?
		}

		if file.is_file() {
			self.def_files.push(file.clone());
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

	fn setup_msvc(
		&self,
		workspace: &mut LuaWorkspace,
		arch: Option<String>,
		platform_type: Option<String>,
		winsdk_version: Option<String>,
	) -> anyhow::Result<HashMap<String, String>>
	{
		let vswhere_path = workspace
			.cache
			.get_dir(&"vswhere".to_string())?
			.join("vswhere.exe");
		if !vswhere_path.exists() {
			download_vswhere(&vswhere_path)?;
		}

		let vswhere_output: String = String::from_utf8(
			Command::new(vswhere_path)
				.args([
					"-latest",
					"-requires",
					"Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
					"-find",
					"VC/Auxiliary/Build",
					"-format",
					"JSON",
				])
				.output()?
				.stdout,
		)?;

		let vswhere_json: Vec<String> = serde_json::from_str(&vswhere_output)?;

		if !vswhere_json.is_empty() {
			let auxpath = dunce::canonicalize(&vswhere_json[0])?;

			let dir = tempdir()?;
			let bat_path = dir.path().join("exec.bat");
			let env_path = dir.path().join("env.txt");

			let mut bat_file = File::create(&bat_path)?;
			writeln!(&bat_file, "@echo off")?;
			writeln!(
				&bat_file,
				"@call \"{}\" {} {} {}",
				auxpath.join("vcvarsall.bat").to_str().unwrap(),
				arch.unwrap_or("x64".to_string()),
				platform_type.unwrap_or("".to_string()),
				winsdk_version.unwrap_or("".to_string())
			)?;
			writeln!(&bat_file, "set > {}", env_path.to_str().unwrap())?;
			bat_file.flush()?;

			Command::new("cmd")
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
				.args(["/C", "@call", bat_path.to_str().unwrap()])
				.status()?;
			let env: String = fs::read_to_string(env_path)?;

			dir.close()?;
			let mut env_variables: HashMap<String, String> = HashMap::new();
			for line in env.lines() {
				let halves: Vec<&str> = line.split('=').collect();
				if halves.len() > 1 {
					env_variables
						.insert(halves[0].to_string(), halves[1].to_string());
				} else {
					env_variables
						.insert(halves[0].to_string(), String::default());
				}
			}

			Ok(env_variables)
		} else {
			Err(anyhow!(&NUMAKE_ERROR.VC_NOT_FOUND))
		}
	}
}

impl TargetTrait for MSVCTarget {
	#[cfg(not(windows))]
	fn build(
		&self,
		_: &LuaWorkspace,
	) -> anyhow::Result<()>
	{
		Err(anyhow!(&NUMAKE_ERROR.MSVC_WINDOWS_ONLY))
	}

	#[cfg(windows)]
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

		let res_dir: PathBuf = parent_workspace
			.workspace
			.join(format!("res/{}", &self.name));

		let msvc_env = self.setup_msvc(
			parent_workspace,
			self.arch.clone(),
			None,
			None,
		)?; // TODO Un-None these

		if !obj_dir.exists() {
			fs::create_dir_all(&obj_dir)?;
		}

		if !out_dir.exists() {
			fs::create_dir_all(&out_dir)?;
		}

		if !res_dir.exists() {
			fs::create_dir_all(&res_dir)?;
		}

		let mut o_files: Vec<String> = Vec::new(); // Can't assume all compilers support wildcards.

		let output: Option<String> = if parent_workspace.output.is_none() {
			self.output.clone()
		} else {
			parent_workspace.output.clone()
		};

		for file in self.files.clone() {
			let mut compiler = Command::new("cl");

			let o_file = obj_dir.join(
				diff_paths(&file, self.workdir.clone())
					.unwrap()
					.to_str()
					.unwrap()
					.to_string() + ".obj",
			);

			if !o_file.parent().unwrap().exists() {
				fs::create_dir_all(o_file.parent().unwrap())?;
			}

			let mut compiler_args = Vec::from([
				"-c".to_string(),
				format!("-Fo{}", o_file.to_str().unwrap()),
			]);

			o_files.push(o_file.to_str().unwrap().to_string());

			for incl in self.include_paths.clone() {
				compiler_args.push(format!("-I{incl}"));
			}

			for define in self.defines.clone() {
				compiler_args.push(format!("-D{define}"));
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
				.envs(&msvc_env)
				.args(&compiler_args)
				.current_dir(&parent_workspace.working_directory)
				.status()?;

			log(&format!("\ncl exited with {}.\n", status), self.quiet);

			if !status.success() {
				log("Aborting...", self.quiet);
				Err(anyhow!(status))?
			}
		}

		for resource_file in self.resources.clone() {
			let mut resource_compiler = Command::new("rc");

			let res_file = res_dir.join(
				diff_paths(&resource_file, self.workdir.clone())
					.unwrap()
					.to_str()
					.unwrap()
					.to_string() + ".res",
			);

			if !res_file.parent().unwrap().exists() {
				fs::create_dir_all(res_file.parent().unwrap())?;
			}

			let mut res_compiler_args = Vec::from([
				"-v".to_string(),
				format!("-fo{}", res_file.to_str().unwrap()),
			]);

			for incl in self.include_paths.clone() {
				res_compiler_args.push(format!("-i{incl}"));
			}

			for define in self.defines.clone() {
				res_compiler_args.push(format!("-d{define}"));
			}

			res_compiler_args
				.push(resource_file.to_str().unwrap_or("ERROR").to_string());

			let status = resource_compiler
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
				.envs(&msvc_env)
				.args(&res_compiler_args)
				.current_dir(&parent_workspace.working_directory)
				.status()?;

			log(&format!("\nrc exited with {}.\n", status), self.quiet);

			if !status.success() {
				log("Aborting...", self.quiet);
				Err(anyhow!(status))?
			}

			let mut cvtres = Command::new("cvtres");

			let rbj_file = obj_dir.join(
				diff_paths(&resource_file, self.workdir.clone())
					.unwrap()
					.to_str()
					.unwrap()
					.to_string() + ".rbj",
			);

			if !rbj_file.parent().unwrap().exists() {
				fs::create_dir_all(rbj_file.parent().unwrap())?;
			}

			let mut cvtres_args =
				Vec::from([format!("/OUT:{}", rbj_file.to_str().unwrap())]);

			o_files.push(rbj_file.to_str().unwrap().to_string());

			for define in self.defines.clone() {
				cvtres_args.push(format!("/DEFINE:{define}"));
			}

			cvtres_args.push(res_file.to_str().unwrap_or("ERROR").to_string());

			let status = cvtres
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
				.envs(&msvc_env)
				.args(&cvtres_args)
				.current_dir(&parent_workspace.working_directory)
				.status()?;

			log(&format!("\ncvtres exited with {}.\n", status), self.quiet);

			if !status.success() {
				log("Aborting...", self.quiet);
				Err(anyhow!(status))?
			}
		}

		let mut linker =
			Command::new(if self.static_lib { "lib" } else { "link" });
		let mut linker_args = Vec::new();

		linker_args.push(format!(
			"/OUT:{}/{}",
			&out_dir.to_str().unwrap_or("ERROR"),
			&output.unwrap_or("out".to_string())
		));

		for path in self.lib_paths.clone() {
			linker_args.push(format!("/LIBPATH:{path}"));
		}

		for def_file in self.def_files.clone() {
			linker_args
				.push(format!("/DEF:{}", def_file.to_str().unwrap_or("ERROR")));
		}

		for flag in self.linker_flags.clone() {
			linker_args.push(flag);
		}

		linker_args.append(&mut o_files);

		linker_args.append(&mut self.libs.clone());

		log(
			&format!(
				"\n{} exited with {}. \n",
				if self.static_lib { "lib" } else { "link" },
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
					.envs(&msvc_env)
					.current_dir(&parent_workspace.working_directory)
					.status()?
			),
			self.quiet,
		);

		self.copy_assets(&out_dir)?;

		Ok(())
	}
}

impl UserData for MSVCTarget
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
			fields.add_field_method_get("arch", |_, this| {
				Ok(this.arch.clone())
			});

			fields.add_field_method_set("arch", |_, this, val: String| {
				this.arch = Some(val);
				Ok(())
			});
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
			fields.add_field_method_get("resource_files", |_, this| {
				let return_val: Vec<String> = this
					.resources
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
				"resource_files",
				|_, this, val: Vec<String>| {
					for path in val {
						to_lua_result(this.add_rc_file(dunce::canonicalize(
							this.workdir.join(path),
						)?))?
					}

					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("rc_flags", |_, this| {
				Ok(this.rc_flags.clone())
			});

			fields.add_field_method_set(
				"rc_flags",
				|_, this, val: Vec<String>| {
					this.rc_flags = val;
					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("def_files", |_, this| {
				let return_val: Vec<String> = this
					.def_files
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
				"def_files",
				|_, this, val: Vec<String>| {
					for path in val {
						to_lua_result(this.add_def_file(dunce::canonicalize(
							this.workdir.join(path),
						)?))?
					}

					Ok(())
				},
			);
		}

		{
			fields.add_field_method_get("static_library", |_, this| {
				Ok(this.static_lib)
			});

			fields.add_field_method_set(
				"static_library",
				|_, this, new_val: bool| {
					this.static_lib = new_val;
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

impl<'lua> FromLua<'lua> for MSVCTarget
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

use std::{
	collections::HashMap,
	fs,
	fs::File,
	io::Write,
	path::{
		Path,
		PathBuf,
	},
	process::Command,
};

use anyhow::anyhow;
use mlua::{
	Error,
	FromLua,
	Lua,
	prelude::LuaError,
	Table,
	UserData,
	UserDataFields,
	UserDataMethods,
	Value,
};
use pathdiff::diff_paths;
use tempfile::tempdir;

use crate::{
	error::{
		NUMAKE_ERROR,
		to_lua_result,
	},
	lua_file::LuaFile,
};

#[derive(Clone)]
pub struct Target
{
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

	pub name: String,

	workdir: PathBuf,
	msvc: bool,
	msvc_arch: Option<String>,
}

impl Target
{
	pub fn new(
		name: String,
		toolset_compiler: Option<String>,
		toolset_linker: Option<String>,
		output: Option<String>,
		workdir: PathBuf,
		msvc: bool,
	) -> anyhow::Result<Self>
	{
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
			workdir,
			name,
			msvc,
			msvc_arch: None,
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

	pub fn add_dir(
		&mut self,
		path_buf: PathBuf,
		recursive: bool,
		filter: &Option<Vec<String>>,
	) -> anyhow::Result<()>
	{
		if !path_buf.starts_with(&self.workdir) {
			Err(LuaError::runtime(NUMAKE_ERROR.PATH_OUTSIDE_WORKING_DIR))?
		}

		for entry in fs::read_dir(path_buf)? {
			let path = dunce::canonicalize(entry?.path())?;
			if path.is_dir() && recursive {
				self.add_dir(path.clone(), true, filter)?
			}
			if path.is_file() {
				if !filter.is_none() {
					if filter.clone().unwrap().contains(
						&path
							.extension()
							.unwrap_or("".as_ref())
							.to_str()
							.unwrap()
							.to_string(),
					) {
						self.add_file(path.clone())?
					}
				} else {
					self.add_file(path.clone())?
				}
			}
		}
		Ok(())
	}

	fn download_vswhere<P: AsRef<Path>>(path: &P) -> anyhow::Result<()>
	{
		let response = reqwest::blocking::get("https://github.com/microsoft/vswhere/releases/latest/download/vswhere.exe")?;
		if response.status().is_success() {
			fs::write(path, response.bytes()?.as_ref())?;
			Ok(())
		} else {
			Err(anyhow!(response.status()))
		}
	}

	fn setup_msvc(
		workspace: &LuaFile,
		arch: Option<String>,
		platform_type: Option<String>,
		winsdk_version: Option<String>,
	) -> anyhow::Result<HashMap<String, String>>
	{
		let remote_dir_str: String = format!(
			// Where the archive will be extracted.
			"{}/remote/vswhere",
			workspace.workspace.to_str().unwrap_or("ERROR")
		);

		let remote_dir = Path::new(&remote_dir_str);
		if !remote_dir.exists() {
			fs::create_dir_all(remote_dir)?;
		}

		let vswhere_path = remote_dir.join("vswhere.exe");
		if !vswhere_path.exists() {
			Self::download_vswhere(&vswhere_path)?;
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

	pub fn build(
		&self,
		parent_workspace: &LuaFile,
	) -> anyhow::Result<()>
	{
		if self.msvc {
			self.build_msvc(parent_workspace)
		} else {
			self.build_generic(parent_workspace)
		}
	}

	fn build_generic(
		&self,
		parent_workspace: &LuaFile,
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
				.args(&compiler_args)
				.current_dir(&parent_workspace.workdir)
				.status()?;

			println!(
				"\n{} exited with {}.\n",
				toolset_compiler.clone().unwrap_or("NULL".to_string()),
				status
			);

			if !status.success() {
				println!("Aborting...");
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

		println!(
			"\n{} exited with {}. \n",
			toolset_linker.clone().unwrap_or("NULL".to_string()),
			linker
				.args(&linker_args)
				.current_dir(&parent_workspace.workdir)
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

	#[cfg(not(windows))]
	fn build_msvc(
		&self,
		_: &LuaFile,
	) -> anyhow::Result<()>
	{
		Err(anyhow!(&NUMAKE_ERROR.MSVC_WINDOWS_ONLY))
	}

	#[cfg(windows)]
	fn build_msvc(
		&self,
		parent_workspace: &LuaFile,
	) -> anyhow::Result<()>
	{
		let obj_dir: PathBuf = parent_workspace
			.workspace
			.join(format!("obj/{}", &self.name));
		let out_dir: PathBuf = parent_workspace
			.workspace
			.join(format!("out/{}", &self.name));

		let msvc_env = Self::setup_msvc(
			parent_workspace,
			self.msvc_arch.clone(),
			None,
			None,
		)?; // TODO Un-None these

		if !obj_dir.exists() {
			fs::create_dir_all(&obj_dir)?;
		}

		if !out_dir.exists() {
			fs::create_dir_all(&out_dir)?;
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
				.envs(&msvc_env)
				.args(&compiler_args)
				.current_dir(&parent_workspace.workdir)
				.status()?;

			println!("\ncl exited with {}.\n", status);

			if !status.success() {
				println!("Aborting...");
				Err(anyhow!(status))?
			}
		}

		let mut linker = Command::new("cl");
		let mut linker_args = Vec::new();

		linker_args.append(&mut o_files);

		linker_args.append(&mut self.libs.clone());

		linker_args.push("/link".to_string());

		linker_args.push(format!(
			"/out:{}/{}",
			&out_dir.to_str().unwrap_or("ERROR"),
			&output.unwrap_or("out".to_string())
		));

		for path in self.lib_paths.clone() {
			linker_args.push(format!("/LIBPATH:{path}"))
		}

		for flag in self.linker_flags.clone() {
			linker_args.push(flag)
		}

		println!(
			"\ncl exited with {}. \n",
			linker
				.args(&linker_args)
				.envs(&msvc_env)
				.current_dir(&parent_workspace.workdir)
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
}
impl UserData for Target
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
				if !this.msvc {
					Ok(this.toolset_compiler.clone())
				} else {
					Ok(Some("MSVC".to_string()))
				}
			});

			fields.add_field_method_set(
				"compiler",
				|_, this, val: Option<String>| {
					if !this.msvc {
						this.toolset_compiler = val;
						Ok(())
					} else {
						Err(mlua::Error::runtime(
							"Cannot modify compiler for MSVC targets!",
						))
					}
				},
			);
		}

		{
			fields.add_field_method_get("linker", |_, this| {
				if !this.msvc {
					Ok(this.toolset_linker.clone())
				} else {
					Ok(Some("MSVC".to_string()))
				}
			});

			fields.add_field_method_set(
				"linker",
				|_, this, val: Option<String>| {
					if !this.msvc {
						this.toolset_linker = val;
						Ok(())
					} else {
						Err(mlua::Error::runtime(
							"Cannot modify linker for MSVC targets!",
						))
					}
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
			fields.add_field_method_get("msvc_arch", |_, this| {
				if this.msvc {
					Ok(this.msvc_arch.clone())
				} else {
					Err(Error::runtime(
						"Cannot get MSVC Architecture for non-MSVC target!",
					))
				}
			});

			fields.add_field_method_set("msvc_arch", |_, this, val: String| {
				if this.msvc {
					this.msvc_arch = Some(val);
					Ok(())
				} else {
					Err(Error::runtime(
						"Cannot set MSVC Architecture for non-MSVC target!",
					))
				}
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
			fields.add_field_method_get("assets", |_, this| {
				Ok(this.assets.clone())
			});

			fields.add_field_method_set("assets", |_, this, val: Table| {
				val.for_each::<String, String>(|old_path, new_path| {
					let path =
						&dunce::canonicalize(this.workdir.join(&old_path))?; // Will automatically error if path doesn't exist.
					if !path.starts_with(&this.workdir) {
						Err(mlua::Error::runtime(
							NUMAKE_ERROR.PATH_OUTSIDE_WORKING_DIR,
						))?
					}

					this.assets.insert(old_path, new_path); // Will validate new path later during build.
					Ok(())
				})
			});
		}
	}

	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M)
	{
		methods.add_method_mut(
			"add_dir",
			|_,
			 this,
			 (path, recursive, filter): (String, bool, Option<Vec<String>>)| {
				to_lua_result(this.add_dir(
					dunce::canonicalize(this.workdir.join(path))?,
					recursive,
					&filter,
				))
			},
		);
	}
}

impl<'lua> FromLua<'lua> for Target
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

use std::{
	collections::HashMap,
	fs,
	path::PathBuf,
	process::{
		Command,
		ExitStatus,
	},
};

use anyhow::anyhow;
use mlua::{
	FromLua,
	Lua,
	prelude::LuaValue,
	Table,
	UserData,
	UserDataFields,
	Value,
};
use pathdiff::diff_paths;
use serde::Serialize;
use which::which;
use crate::lib::error::NuMakeError::{AddFileIsDirectory, AssetCopyPathOutsideWorkingDirectory, PathOutsideWorkingDirectory};
use crate::lib::target::{TargetTrait, VSCodeProperties};
use crate::lib::ui::NumakeUI;
use crate::lib::util::{get_gcc_includes, to_lua_result};
use crate::lib::workspace::LuaWorkspace;

#[derive(Clone, Serialize)]
pub struct MinGWTarget
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
	vscode_properties: VSCodeProperties,

	#[serde(skip_serializing)]
	ui: NumakeUI,

	pub resources: Vec<PathBuf>,
	pub def_files: Vec<PathBuf>,
	pub windres_flags: Vec<String>,
	arch: Option<String>,
	static_lib: bool,
	lang: String,
}

impl MinGWTarget
{
	pub fn new(
		name: String,
		output: Option<String>,
		workdir: PathBuf,
		ui: NumakeUI,
	) -> anyhow::Result<Self>
	{
		Ok(MinGWTarget {
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
			ui,
			resources: Vec::new(),
			def_files: Vec::new(),
			windres_flags: Vec::new(),
			static_lib: false,
			name,
			lang: "C++".to_string(),
			vscode_properties: VSCodeProperties::default(),
		})
	}

	pub fn add_file(
		&mut self,
		file: PathBuf,
	) -> anyhow::Result<()>
	{
		if !file.starts_with(&self.workdir) {
			return Err(anyhow!(PathOutsideWorkingDirectory))
		}

		if file.is_file() {
			self.files.push(file.clone());
		} else {
			return Err(anyhow!(AddFileIsDirectory))
		}
		Ok(())
	}

	pub fn add_rc_file(
		&mut self,
		file: PathBuf,
	) -> anyhow::Result<()>
	{
		if !file.starts_with(&self.workdir) {
			return Err(anyhow!(PathOutsideWorkingDirectory))
		}

		if file.is_file() {
			self.resources.push(file.clone());
		} else {
			return Err(anyhow!(AddFileIsDirectory))
		}
		Ok(())
	}

	pub fn add_def_file(
		&mut self,
		file: PathBuf,
	) -> anyhow::Result<()>
	{
		if !file.starts_with(&self.workdir) {
			return Err(anyhow!(PathOutsideWorkingDirectory))
		}

		if file.is_file() {
			self.def_files.push(file.clone());
		} else {
			return Err(anyhow!(AddFileIsDirectory))
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
				Err(anyhow!(AssetCopyPathOutsideWorkingDirectory))?
			} else {
				fs::copy(key, copy_path)?;
			}
		}

		Ok(())
	}
}

impl TargetTrait for MinGWTarget
{
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

		let mingw = format!(
			"{}-w64-mingw32-",
			self.arch.clone().unwrap_or("x86_64".to_string())
		);

		// COMPILATION STEP
		for file in self.files.clone() {
			let mut compiler = Command::new(
				mingw.clone()
					+ if self.lang.to_lowercase() == "c++" {
						"g++"
					} else {
						"gcc"
					},
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

			self.execute(
				compiler
					.args(&compiler_args)
					.current_dir(&parent_workspace.working_directory),
			)?;
		}

		// RESOURCE FILE HANDLING
		for resource_file in self.resources.clone() {
			let mut resource_compiler = Command::new(mingw.clone() + "windres");

			let coff_file = res_dir.join(
				diff_paths(&resource_file, self.workdir.clone())
					.unwrap()
					.to_str()
					.unwrap()
					.to_string() + ".o",
			);

			if !coff_file.parent().unwrap().exists() {
				fs::create_dir_all(coff_file.parent().unwrap())?;
			}

			let mut res_compiler_args = Vec::from([
				"-v".to_string(),
				resource_file.to_str().unwrap_or("ERROR").to_string(),
				"-OCOFF".to_string(),
			]);

			for incl in self.include_paths.clone() {
				res_compiler_args.push(format!("-I{incl}"));
			}

			for define in self.defines.clone() {
				res_compiler_args.push(format!("-D{define}"));
			}

			res_compiler_args
				.push(format!("-o{}", coff_file.to_str().unwrap()));

			self.execute(
				resource_compiler
					.args(&res_compiler_args)
					.current_dir(&parent_workspace.working_directory),
			)?;

			o_files.push(coff_file.to_str().unwrap().to_string());
		}

		if !self.static_lib {
			// LINKING STEP
			let mut linker = Command::new(
				mingw.clone()
					+ if self.lang.to_lowercase() == "c++" {
						"g++"
					} else {
						"gcc"
					},
			);
			let mut linker_args = Vec::new();

			linker_args.append(&mut o_files);

			for def_file in self.def_files.clone() {
				linker_args.push(def_file.to_str().unwrap().to_string());
			}

			for path in self.lib_paths.clone() {
				linker_args.push(format!("-L{path}"))
			}

			for lib in self.libs.clone() {
				linker_args.push(format!("-l{lib}"))
			}

			for flag in self.compiler_flags.clone() {
				linker_args.push(flag)
			}

			for flag in self.linker_flags.clone() {
				linker_args.push("-Wl,".to_string() + &flag)
			}

			linker_args.push(format!(
				"-o{}/{}",
				&out_dir.to_str().unwrap_or("ERROR"),
				&output.unwrap_or("out".to_string())
			));

			self.execute(
				linker
					.args(&linker_args)
					.current_dir(&parent_workspace.working_directory),
			)?;
		} else {
			let mut linker = Command::new(mingw.clone() + "ar");
			let mut linker_args = Vec::from([
				"rcs".to_string(),
				format!(
					"{}/{}",
					&out_dir.to_str().unwrap_or("ERROR"),
					&output.unwrap_or("out".to_string())
				),
			]);

			linker_args.append(&mut o_files);

			for def_file in self.def_files.clone() {
				linker_args.push(def_file.to_str().unwrap().to_string());
			}

			self.execute(
				linker
					.args(&linker_args)
					.current_dir(&parent_workspace.working_directory),
			)?;
		}

		self.copy_assets(&out_dir)?;

		Ok(())
	}

	fn execute(
		&self,
		cmd: &mut Command,
	) -> anyhow::Result<ExitStatus>
	{
		let result = cmd.output();
		if result.is_err() {
			let err = result.err().unwrap();
			Err(anyhow!(format!(
				"Error trying to execute {}! {}",
				cmd.get_program().to_str().unwrap(),
				err
			)))
		} else {
			let output = result.ok().unwrap();
			let stderr = String::from_utf8_lossy(&output.stderr).to_string();

			if output.status.success() {
				if !stderr.is_empty() {
					self.ui
						.progress_manager
						.println(self.ui.format_warn(stderr.clone()))?;
				}

				self.ui.progress_manager.println(self.ui.format_ok(
					format!(
						"{} exited with {}",
						cmd.get_program().to_str().unwrap(),
						output.status
					),
				))?;
				Ok(output.status)
			} else {
				self.ui.progress_manager.println(self.ui.format_err(
					format!(
						"{} exited with {}",
						cmd.get_program().to_str().unwrap(),
						output.status
					),
				))?;
				Err(anyhow!(stderr))
			}
		}
	}

	fn set_vscode_props(&mut self, lua_workspace: &mut LuaWorkspace) -> anyhow::Result<VSCodeProperties>
	{
		self.vscode_properties = VSCodeProperties {
			compiler_path: which(format!(
				"{}-w64-mingw32-{}",
				self.arch.clone().unwrap_or("x86_64".to_string()),
				if self.lang.to_lowercase() == "c++" {
					"g++"
				} else {
					"gcc"
				}
			))?
			.to_str()
			.unwrap()
			.to_string(),

			default_includes: get_gcc_includes(format!(
				"{}-w64-mingw32-{}",
				self.arch.clone().unwrap_or("x86_64".to_string()),
				if self.lang.to_lowercase() == "c++" {
					"g++"
				} else {
					"gcc"
				}
			))?,

			intellisense_mode: format!(
				"{}-gcc-{}",
				std::env::consts::OS,
				if self.arch == Some("i686".to_string()) {
					"x86".to_string()
				} else if self.arch == Some("x86_64".to_string()) {
					"x64".to_string()
				} else {
					self.arch.clone().unwrap_or("${default}".into())
				}
			),
		};

		Ok(self.vscode_properties.clone())
	}
}

impl UserData for MinGWTarget
{
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F)
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
			fields
				.add_field_method_get("arch", |_, this| Ok(this.arch.clone()));

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
			fields.add_field_method_get("windres_flags", |_, this| {
				Ok(this.windres_flags.clone())
			});

			fields.add_field_method_set(
				"windres_flags",
				|_, this, val: Vec<String>| {
					this.windres_flags = val;
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
							PathOutsideWorkingDirectory,
						))?
					}

					this.assets.insert(path, val); // Will validate new path later during build.

					Ok(())
				})
			});
		}
	}
}

impl FromLua for MinGWTarget
{
	fn from_lua(
		value: LuaValue,
		_: &Lua,
	) -> mlua::Result<Self>
	{
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

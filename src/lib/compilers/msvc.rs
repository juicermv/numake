use crate::lib::data::environment::Environment;
use crate::lib::data::project::Project;
use crate::lib::data::project_type::ProjectType;
use crate::lib::data::source_file_type::SourceFileType;
use crate::lib::util::error::NuMakeError::{
	MsvcWindowsOnly, VcNotFound,
};
use crate::lib::util::download_vswhere;
use anyhow::anyhow;
use mlua::{prelude::LuaValue, FromLua, Lua, UserData, UserDataMethods, Value};
use pathdiff::diff_paths;
use serde::Serialize;
use std::{
	collections::HashMap,
	fs,
	fs::File,
	io::Write,
	path::PathBuf,
	process::{Command, ExitStatus},
};
use tempfile::tempdir;
use crate::lib::util::ui::NumakeUI;

#[derive(Clone, Serialize)]
pub struct MSVC {
	#[serde(skip)]
	environment:  Environment,
	#[serde(skip)]
	ui:  NumakeUI,
}

impl MSVC {
	pub fn new(
		environment:  Environment,
		ui:  NumakeUI,
	) -> Self {
		MSVC { environment, ui }
	}

	 fn setup_msvc(
		&self,
		arch: Option<String>,
		platform_type: Option<String>,
		winsdk_version: Option<String>,
	) -> anyhow::Result<HashMap<String, String>> {
		let vswhere_path = (self.environment).numake_directory
			.join("vswhere.exe");
		if !vswhere_path.exists() {
			download_vswhere(&vswhere_path)?;
		}

		let vswhere_output: String = String::from_utf8_lossy(
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
				.stdout
				.as_slice(),
		)
		.to_string();

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

			self.execute(Command::new("cmd").args([
				"/C",
				"@call",
				bat_path.to_str().unwrap(),
			]))?;

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
			Err(anyhow!(VcNotFound))
		}
	}

	 fn compilation_step(
		&self,
		project: &Project,
		working_directory: &PathBuf,
		obj_dir: &PathBuf,
		msvc_env: &HashMap<String, String>,
		o_files: &mut Vec<String>,
	) -> anyhow::Result<()> {
		// COMPILATION STEP
		for file in project.source_files.get(&SourceFileType::Code) {
			let mut compiler = Command::new("CL");

			let o_file = obj_dir.join(
				diff_paths(&file, &(self.environment).numake_directory)
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

			for incl in project.include_paths.clone() {
				compiler_args.push(format!("-I{incl}"));
			}

			for define in project.defines.clone() {
				compiler_args.push(format!("-D{define}"));
			}

			for flag in project.compiler_flags.clone() {
				compiler_args.push(flag)
			}

			compiler_args.push(file.to_str().unwrap_or("ERROR").to_string());

			self.execute(
				compiler
					.envs(msvc_env)
					.args(&compiler_args)
					.current_dir(working_directory),
			)?;
		}

		Ok(())
	}

	 fn resource_step(
		&self,
		project: &Project,
		working_directory: &PathBuf,
		obj_dir: &PathBuf,
		res_dir: &PathBuf,
		msvc_env: &HashMap<String, String>,
		o_files: &mut Vec<String>,
	) -> anyhow::Result<()> {
		// RESOURCE FILE HANDLING
		for resource_file in project.source_files.get(&SourceFileType::Resource)
		{
			let mut resource_compiler = Command::new("RC");

			let res_file = res_dir.join(
				diff_paths(&resource_file, &(self.environment).numake_directory)
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

			for incl in project.include_paths.clone() {
				res_compiler_args.push(format!("-i{incl}"));
			}

			for define in project.defines.clone() {
				res_compiler_args.push(format!("-d{define}"));
			}

			res_compiler_args
				.push(resource_file.to_str().unwrap_or("ERROR").to_string());

			self.execute(
				resource_compiler
					.envs(msvc_env)
					.args(&res_compiler_args)
					.current_dir(working_directory),
			)?;

			// TURN RES FILES INTO OBJECTS
			let mut cvtres = Command::new("CVTRES");

			let rbj_file = obj_dir.join(
				diff_paths(&resource_file, &(self.environment).numake_directory)
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

			for define in project.defines.clone() {
				cvtres_args.push(format!("/DEFINE:{define}"));
			}

			cvtres_args.push(res_file.to_str().unwrap_or("ERROR").to_string());

			self.execute(
				cvtres
					.envs(msvc_env)
					.args(&cvtres_args)
					.current_dir(working_directory),
			)?;
		}

		Ok(())
	}

	 fn linking_step(
		&self,
		project: &Project,
		output: &String,
		working_directory: &PathBuf,
		out_dir: &PathBuf,
		msvc_env: &HashMap<String, String>,
		o_files: &mut Vec<String>,
	) -> anyhow::Result<()> {
		// LINKING STEP
		let mut linker = Command::new(match project.project_type {
			ProjectType::StaticLibrary => "LIB",
			_ => "LINK",
		});

		let mut linker_args = Vec::new();

		linker_args.push(format!(
			"/OUT:{}/{}",
			&out_dir.to_str().unwrap_or("ERROR"),
			output
		));

		for path in project.lib_paths.clone() {
			linker_args.push(format!("/LIBPATH:{path}"));
		}

		for def_file in
			project.source_files.get(&SourceFileType::ModuleDefinition)
		{
			linker_args
				.push(format!("/DEF:{}", def_file.to_str().unwrap_or("ERROR")));
		}

		for flag in project.linker_flags.clone() {
			linker_args.push(flag);
		}

		linker_args.append(o_files);

		linker_args.append(&mut project.libs.clone());

		self.execute(
			linker
				.args(&linker_args)
				.envs(msvc_env)
				.current_dir(working_directory),
		)?;

		Ok(())
	}

	#[cfg(not(windows))]
	 fn build(
		&self,
		_: &Project
	) -> anyhow::Result<()> {
		Err(anyhow!(MsvcWindowsOnly))
	}

	#[cfg(windows)]
	 fn build(
		&self,
		project: &Project,
	) -> anyhow::Result<()> {
		let obj_dir: PathBuf = (self.environment).numake_directory
			.join(format!("obj/{}", project.name));
		let out_dir: PathBuf = (self.environment).numake_directory
			.join(format!("out/{}", project.name));

		let res_dir: PathBuf = (self.environment).numake_directory
			.join(format!("res/{}", project.name));

		let msvc_env = self.setup_msvc(
			project.arch.clone(),
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

		let mut o_files: Vec<String> = Vec::new();

		let working_directory = &(self.environment).project_directory;

		self.compilation_step(
			project,
			&working_directory,
			&obj_dir,
			&msvc_env,
			&mut o_files,
		)?;

		self.resource_step(
			project,
			&working_directory,
			&obj_dir,
			&res_dir,
			&msvc_env,
			&mut o_files,
		)?;

		self.linking_step(
			project,
			&project.output.clone().unwrap_or("out".to_string()),
			&working_directory,
			&out_dir,
			&msvc_env,
			&mut o_files,
		)?;

		project.copy_assets(&(self.environment).numake_directory, &out_dir)?;

		Ok(())
	}

	// TODO move this out of here
	 fn execute(
		&self,
		cmd: &mut Command,
	) -> anyhow::Result<ExitStatus> {
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
			let stdout = String::from_utf8_lossy(&output.stdout).to_string();

			if output.status.success() {
				(self.ui).progress_manager.println(
					if stdout.contains(": warning ") {
						(self.ui).format_warn(stdout.clone())
					} else {
						(self.ui).format_ok(stdout.clone())
					},
				)?;

				(self.ui).progress_manager.println((self.ui).format_ok(
					format!(
						"{} exited with {}",
						cmd.get_program().to_str().unwrap(),
						output.status
					),
				))?;
				Ok(output.status)
			} else {
				(self.ui).progress_manager.println((self.ui).format_err(
					format!(
						"{} exited with {}",
						cmd.get_program().to_str().unwrap(),
						output.status
					),
				))?;
				Err(anyhow!(stdout))
			}
		}
	}
}

impl UserData for MSVC {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut("build", |_,this,project: Project|  {
			match this.build(&project) {
				Ok(_) => Ok(()),
				Err(err) => { Err(mlua::Error::external(err))}
			}
		})
	}
}

impl FromLua for MSVC {
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

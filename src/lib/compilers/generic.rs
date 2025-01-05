use std::{
	collections::HashMap,
	env, fs,
	path::PathBuf,
	process::{Command, ExitStatus},
};

use crate::lib::compilers::mingw::MinGW;
use crate::lib::data::environment::Environment;
use crate::lib::data::project::Project;
use crate::lib::data::source_file_type::SourceFileType;
use crate::lib::error::NuMakeError::{
	AddFileIsDirectory, AssetCopyPathOutsideWorkingDirectory,
	PathOutsideWorkingDirectory, ToolsetCompilerNull, ToolsetLinkerNull,
};
use crate::lib::ui::NumakeUI;
use crate::lib::util::{get_gcc_includes, to_lua_result};
use anyhow::anyhow;
use mlua::{
	prelude::LuaValue, FromLua, Lua, Table, UserData, UserDataFields,
	UserDataMethods, Value,
};
use pathdiff::diff_paths;
use serde::Serialize;
use which::which;

#[derive(Clone, Serialize)]
pub struct Generic {
	#[serde(skip)]
	environment:  Environment,
	#[serde(skip)]
	ui:  NumakeUI,
}

impl Generic {
	pub fn new(
		environment:  Environment,
		ui:  NumakeUI,
	) -> Self {
		Generic { environment, ui }
	}

	unsafe fn compile_step(
		&self,
		project: &Project,
		toolset_compiler: &String,
		obj_dir: &PathBuf,
		o_files: &mut Vec<String>,
	) -> anyhow::Result<()> {
		for file in project.source_files.get(&SourceFileType::Code) {
			let mut compiler = Command::new(toolset_compiler);

			let o_file = obj_dir.join(
				diff_paths(&file, &(self.environment).project_directory)
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

			for incl in project.include_paths.clone() {
				compiler_args.push(format!("-I{incl}"))
			}

			for define in project.defines.clone() {
				compiler_args.push(format!("-D{define}"))
			}

			for flag in project.compiler_flags.clone() {
				compiler_args.push(flag)
			}

			compiler_args.push(file.to_str().unwrap_or("ERROR").to_string());

			self.execute(
				compiler
					.args(&compiler_args)
					.current_dir(&(self.environment).project_directory),
			)?;
		}

		Ok(())
	}

	unsafe fn linking_step(
		&self,
		project: &Project,
		toolset_linker: &String,
		out_dir: &PathBuf,
		output: &String,
		o_files: &mut Vec<String>,
	) -> anyhow::Result<()> {
		let mut linker = Command::new(toolset_linker);
		let mut linker_args = Vec::new();

		linker_args.append(o_files);

		for path in project.lib_paths.clone() {
			linker_args.push(format!("-L{path}"))
		}

		for lib in project.libs.clone() {
			linker_args.push(format!("-l{lib}"))
		}

		for flag in project.linker_flags.clone() {
			linker_args.push(flag)
		}

		linker_args.push(format!(
			"-o{}/{}",
			&out_dir.to_str().unwrap_or("ERROR"),
			&output
		));

		self.execute(
			linker
				.args(&linker_args)
				.current_dir(&(self.environment).project_directory),
		)?;

		Ok(())
	}

	unsafe fn build(
		&self,
		toolset_compiler: &String,
		toolset_linker: &String,
		project: &Project,
	) -> anyhow::Result<()> {
		let obj_dir: PathBuf = (self.environment)
			.numake_directory
			.join(format!("obj/{}", project.name));
		let out_dir: PathBuf = (self.environment)
			.numake_directory
			.join(format!("out/{}", project.name));

		if !obj_dir.exists() {
			fs::create_dir_all(&obj_dir)?;
		}

		if !out_dir.exists() {
			fs::create_dir_all(&out_dir)?;
		}

		let mut o_files: Vec<String> = Vec::new(); // Can't assume all compilers support wildcards.

		self.compile_step(project, toolset_compiler, &obj_dir, &mut o_files)?;
		self.linking_step(
			project,
			toolset_linker,
			&out_dir,
			&project.output.clone().unwrap_or("out".to_string()),
			&mut o_files,
		)?;

		project.copy_assets(&(self.environment).numake_directory, &out_dir)?;

		Ok(())
	}

	unsafe fn execute(
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
			let stderr = String::from_utf8_lossy(&output.stderr).to_string();

			if output.status.success() {
				if !stderr.is_empty() {
					(self.ui)
						.progress_manager
						.println((self.ui).format_warn(stderr.clone()))?;
				}

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
				Err(anyhow!(stderr))
			}
		}
	}
}

impl UserData for Generic {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut(
			"build",
			|_,
			 this,
			 (project, compiler, linker): (Project, String, String)| unsafe {
				match this.build(&compiler, &linker, &project) {
					Ok(_) => Ok(()),
					Err(err) => Err(mlua::Error::runtime(err)),
				}
			},
		)
	}
}

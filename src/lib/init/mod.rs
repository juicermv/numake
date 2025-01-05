use crate::lib::cli::sub_commands::SubCommands;
use crate::lib::cli::Cli;
use crate::lib::data::environment::Environment;
use crate::lib::runtime::Runtime;
use clap::{Parser, Subcommand};
use mlua::prelude::LuaResult;
use std::env;

pub struct Init {}

impl Init {
	pub fn run() -> LuaResult<()> {
		let cmd = Self::get_subcommand(&Cli::parse());
		let env = Self::init_environment(&cmd)?;
		env::set_current_dir(&env.project_directory)?;
		let mut runtime = Self::init_runtime(&cmd, env.clone())?;
		runtime.execute_script(
			&env.project_file
				.to_str()
				.unwrap_or("ERROR")
				.to_string(),
		)?;

		match cmd {
			SubCommands::Build(args) => {
				runtime.execute_task(&*args.task)
			}

			SubCommands::List(_) => {
				println!(
					"Available Tasks: {}",
					runtime.get_tasks().join(", ")
				);
				Ok(())
			}
		}
	}

	fn init_runtime(
		cmd: &SubCommands,
		env: Environment,
	) -> anyhow::Result<Runtime> {
		let quiet = Self::check_quiet(cmd);

		Ok(Runtime::new(env, quiet))
	}

	fn get_subcommand(args: &Cli) -> SubCommands {
		args.command.clone()
	}

	fn init_environment(command: &SubCommands) -> anyhow::Result<Environment> {
		let mut project_dir_str = "";
		let mut project_file_str = "";

		match command {
			SubCommands::Build(args) => {
				project_dir_str = args.workdir.as_str();
				project_file_str = args.file.as_str()
			}

			SubCommands::List(args) => {
				project_dir_str = args.workdir.as_str();
				project_file_str = args.file.as_str()
			}
		}

		let project_directory = dunce::canonicalize(project_dir_str)?;
		let project_file = project_directory.join(project_file_str);
		let numake_directory = project_directory.join("numake");

		Ok(Environment {
			project_file,
			project_directory,
			numake_directory,
		})
	}

	fn check_quiet(command: &SubCommands) -> bool {
		match command {
			SubCommands::Build(args) => args.quiet,

			SubCommands::List(args) => args.quiet,
		}
	}
}

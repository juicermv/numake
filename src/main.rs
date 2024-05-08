// @formatter:on

/*
	TODO: Optimization, Refactoring, Error Handling. THIS IS A WIP!
*/

use clap::Parser;
use console::{
	Emoji,
	style,
};
use mlua::Lua;

use crate::{
	cli_args::{
		Cli,
		SubCommands,
	},
	lua_workspace::LuaWorkspace,
};

mod cache;
mod cli_args;
mod error;
mod generic_target;
mod lua_workspace;
mod msvc_target;
mod target;
mod ui;
mod util;

fn run() -> anyhow::Result<()>
{
	let cli = Cli::parse();
	let lua = Lua::new();
	lua.enable_jit(true);
	lua.sandbox(true)?;

	match &cli.command {
		SubCommands::Build(args) => {
			let mut proj = LuaWorkspace::new(args)?;
			proj.process(&lua)?;
			proj.build()?;
		}

		SubCommands::Inspect(args) => {
			let mut proj = LuaWorkspace::new_inspect(args)?;
			proj.process(&lua)?;

			println!("{}", serde_json::to_string_pretty(&proj)?);
		}

		SubCommands::List(args) => {
			let mut proj = LuaWorkspace::new_dummy(args)?;
			proj.process(&lua)?;
			println!("\nAvailable targets: {}", proj.list_targets()?);
		}
	}

	Ok(())
}

#[cfg(not(test))]
fn main()
{
	let result = run();
	if result.is_err() {
		println!(
			"{}",
			style(format!(
				"{} {}",
				Emoji("⛔", "ERROR!"),
				result.err().unwrap()
			))
				.red()
				.bold()
		)
	}
}

#[cfg(test)]
mod tests
{
	use mlua::Lua;

	use crate::{
		cli_args::NuMakeArgs,
		lua_workspace::LuaWorkspace,
	};

	#[test]
	fn gcc_build() -> anyhow::Result<()>
	{
		let args: NuMakeArgs = NuMakeArgs {
			target: "gcc".to_string(),
			toolset_compiler: None,
			toolset_linker: None,
			file: "test.lua".to_string(),
			output: None,
			workdir: "examples/test".to_string(),
			arguments: Some(vec![]),
			quiet: false,
		};

		let mut proj = LuaWorkspace::new(&args)?;
		proj.process(&Lua::new())?;
		proj.build()?;

		let mut test_exec =
			std::process::Command::new("examples/test/.numake/out/gcc/test");
		assert_eq!(test_exec.status()?.code(), Some(0));
		Ok(())
	}

	#[test]
	fn mingw_build() -> anyhow::Result<()>
	{
		let args: NuMakeArgs = NuMakeArgs {
			target: "mingw".to_string(),
			toolset_compiler: None,
			toolset_linker: None,
			file: "test.lua".to_string(),
			output: None,
			workdir: "examples/test".to_string(),
			arguments: Some(vec![]),
			quiet: false,
		};

		let mut proj = LuaWorkspace::new(&args)?;
		proj.process(&Lua::new())?;
		proj.build()?;

		let mut test_exec = std::process::Command::new(
			"examples/test/.numake/out/mingw/test.exe",
		);
		assert_eq!(test_exec.status()?.code(), Some(0));
		Ok(())
	}
}

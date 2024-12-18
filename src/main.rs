// @formatter:on

/*
	TODO: Optimization, Refactoring, Error Handling. THIS IS A WIP!
*/
use crate::lib::cli_args::{Cli, SubCommands};
use crate::lib::workspace::LuaWorkspace;
use anyhow::anyhow;
use clap::Parser;
use console::Term;
use mlua::Lua;
use std::fmt;

mod lib;

fn run() -> anyhow::Result<()> {
	let cli = Cli::parse();
	let lua = Lua::new();
	lua.enable_jit(true);
	match lua.sandbox(true) {
		Err(e) => return Err(anyhow!(e.to_string())),
		Ok(()) => {}
	}

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
			println!("\nAvailable target: {}", proj.list_targets()?);
		}
	}

	Ok(())
}

#[cfg(not(test))]
fn main() -> anyhow::Result<()> {
	let result = run();
	result
}

#[cfg(test)]
mod tests {
	use crate::lib::cli_args::NuMakeArgs;
	use crate::lib::workspace::LuaWorkspace;
	use mlua::Lua;

	#[test]
	fn gcc_build() -> anyhow::Result<()> {
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

		let mut test_exec = std::process::Command::new(".numake/out/gcc/test");
		assert_eq!(test_exec.status()?.code(), Some(0));
		Ok(())
	}

	#[test]
	fn mingw_build() -> anyhow::Result<()> {
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

		let mut test_exec =
			std::process::Command::new(".numake/out/mingw/test.exe");
		assert_eq!(test_exec.status()?.code(), Some(0));
		Ok(())
	}

	// Does not at the moment work on GH actions for some unknown reason
	#[test]
	fn msvc_build() -> anyhow::Result<()> {
		let args: NuMakeArgs = NuMakeArgs {
			target: "msvc".to_string(),
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

		let test_exec =
			std::process::Command::new(".numake/out/msvc/test.exe").output()?;
		println!("{}", String::from_utf8_lossy(test_exec.stdout.as_slice()));
		println!("{}", String::from_utf8_lossy(test_exec.stderr.as_slice()));

		assert_eq!(test_exec.status.code(), Some(0));
		Ok(())
	}
}

// @formatter:on

/*
	TODO: Optimization, Refactoring, Error Handling. THIS IS A WIP!
*/

use clap::Parser;
use mlua::Lua;

use crate::{
	config::{
		Cli,
		Subcommands,
	},
	lua_file::LuaFile,
};

mod config;
mod error;
mod lua_file;
mod target;

#[cfg(not(test))]
fn main() -> anyhow::Result<()>
{
	let cli = Cli::parse();
	let lua = Lua::new();
	lua.enable_jit(true);
	lua.sandbox(true)?;
	println!("Warming up...");

	match &cli.command {
		Subcommands::Build(args) => {
			let mut proj = LuaFile::new(args)?;
			proj.process(&lua)?;
			proj.build()?;
		}

		//Subcommands::Inspect(_) => {}
		Subcommands::List(args) => {
			let mut proj = LuaFile::new_dummy(args)?;
			proj.process(&lua)?;
			println!("{}", proj.list_targets()?);
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests
{
	use std::{
		fs::File,
		io::Write,
	};

	use mlua::Lua;
	use tempfile::tempdir;

	use crate::{
		config::NuMakeArgs,
		lua_file::LuaFile,
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
		};

		let mut proj = LuaFile::new(&args)?;
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
		};

		let mut proj = LuaFile::new(&args)?;
		proj.process(&Lua::new())?;
		proj.build()?;

		let mut test_exec = std::process::Command::new(
			"examples/test/.numake/out/mingw/test.exe",
		);
		assert_eq!(test_exec.status()?.code(), Some(0));
		Ok(())
	}

	#[test]
	fn vcvars() -> anyhow::Result<()>
	{
		let dir = tempdir()?;
		let path = dir.path().join("exec.bat");
		let mut file = File::create(&path)?;
		writeln!(&file, "@echo off")?;
		writeln!(&file, "@call \"C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Auxiliary\\Build\\vcvarsall.bat\" x64")?;
		writeln!(&file, "@echo -$-")?;
		writeln!(&file, "set")?;
		file.flush()?;

		let mut process = std::process::Command::new("cmd")
			.args(["/C", "@call", path.to_str().unwrap()])
			.output()?;
		println!(
			"OUTPUT: {}",
			String::from_utf8(process.stdout)?
				.split("-$-")
				.collect::<Vec<&str>>()[1]
		);

		Ok(())
	}
}

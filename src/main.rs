// @formatter:on

/*
	TODO: Optimization, Refactoring, Error Handling. THIS IS A WIP!
*/
use crate::lib::cli_args::{Cli, SubCommands};
use anyhow::anyhow;
use clap::Parser;
use console::Term;
use mlua::Lua;
use mlua::prelude::LuaResult;
use crate::lib::data::environment::Environment;
use crate::lib::runtime::Runtime;

mod lib;

fn run() -> LuaResult<()> {
	let mut runtime = Runtime::new(
		Environment {
			numake_directory: "./.numake".parse().unwrap(),
			project_directory: ".".parse().unwrap(),
		}
	);

	runtime.execute_script("tasks:create('hello', function()print('hello world!')end)")?;
	runtime.execute_task("hello")?;

	Ok(())
}

#[cfg(not(test))]
fn main() -> LuaResult<()> {
	let result = run();
	result
}

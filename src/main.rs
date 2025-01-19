// @formatter:on

/*
	TODO: Optimization, Refactoring, Error Handling. THIS IS A WIP!
*/
use std::process::ExitCode;
use mlua::prelude::LuaResult;
use crate::lib::init::Init;

mod lib;


#[cfg(not(test))]
fn main() -> ExitCode {
	Init::run()
}

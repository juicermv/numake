// @formatter:on

/*
	TODO: Optimization, Refactoring, Error Handling. THIS IS A WIP!
*/
use clap::Parser;
use mlua::prelude::LuaResult;
use crate::lib::data::environment::Environment;
use crate::lib::init::Init;
use crate::lib::runtime::Runtime;

mod lib;


#[cfg(not(test))]
fn main() -> LuaResult<()> {
	Init::run()
}

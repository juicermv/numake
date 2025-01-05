// @formatter:on

/*
	TODO: Optimization, Refactoring, Error Handling. THIS IS A WIP!
*/

mod lib;


#[cfg(not(test))]
fn main() -> LuaResult<()> {
	Init::run()
}

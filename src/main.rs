// @formatter:on

/*
    TODO: Optimization, Refactoring, Error Handling. THIS IS A WIP!
*/

use crate::config::{Cli, Subcommands};
use crate::lua_file::LuaFile;
use clap::Parser;
use mlua::Lua;
use std::time::SystemTime;

mod config;
mod error;
mod lua_file;
mod target;

#[cfg(not(test))]
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let lua = Lua::new();
    lua.enable_jit(true);
    lua.sandbox(true)?;

    match &cli.command {
        Subcommands::Build(args) => {
            let mut proj = LuaFile::new(args)?;
            proj.process(&lua)?;
            let now = SystemTime::now();
            println!("Building target {}...", &args.target);
            proj.build()?;
            println!(
                "\n\nBuilding target {} done in {}ms!",
                &args.target,
                now.elapsed()?.as_millis()
            );
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
mod tests {
    use crate::config::NuMakeArgs;
    use crate::lua_file::LuaFile;
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
            msvc: false,
            arguments: Some(vec![]),
        };

        let mut proj = LuaFile::new(&args)?;
        proj.process(&Lua::new())?;
        proj.build()?;

        let mut test_exec = std::process::Command::new("examples/test/.numake/out/gcc/test");
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
            msvc: false,
            arguments: Some(vec![]),
        };

        let mut proj = LuaFile::new(&args)?;
        proj.process(&Lua::new())?;
        proj.build()?;

        let mut test_exec = std::process::Command::new("examples/test/.numake/out/mingw/test.exe");

        assert_eq!(test_exec.status()?.code(), Some(0));
        Ok(())
    }
}

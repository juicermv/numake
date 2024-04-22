// @formatter:on

/*
    TODO: Optimization, Refactoring, Error Handling. THIS IS A WIP!
*/

mod numake_file;

mod config;
mod error;

use std::io;
use std::io::Write;
use clap::Parser;
use std::time::SystemTime;
use crate::config::{Cli, Subcommands};
use crate::numake_file::Project;

#[cfg(not(test))]
fn main() -> anyhow::Result<()> {
    let now = SystemTime::now();

    let cli = Cli::parse();
    match &cli.command {
        Subcommands::Build(args) => {
            let mut proj = Project::new(args)?;
            proj.setup_lua_vals()?;
            proj.process()?;
            proj.build()?;
            println!(
                "\n\nNuMake done in {}ms!",
                now.elapsed()?.as_millis()
            );
        }

        Subcommands::Inspect(args) => {
            let mut proj = Project::new(args)?;
            proj.setup_lua_vals()?;
            proj.process()?;
            io::stdout()
                .write_all(
                    format!(
                        "{{\n\t'include': \n\t[ {} \n\t]\n}}",
                        proj.include_paths
                            .into_iter()
                            .map(|item| { format!("\n\t\t'{}', ", item) })
                            .collect::<String>()
                    )
                    .as_bytes(),
                )?
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use crate::config::NuMakeArgs;
    use crate::numake_file::Project;

    #[test]
    fn gcc_build () -> anyhow::Result<()> {
        let args: NuMakeArgs = NuMakeArgs{
            target: env!("TARGET").to_string().to_string(),
            toolset_compiler: None,
            toolset_linker: None,
            file: "test.numake".to_string(),
            output: "test".to_string(),
            workdir: "examples/test".to_string(),
            msvc: None,
            arguments: None
        };

        let mut proj = Project::new(&args)?;
        proj.setup_lua_vals()?;
        proj.process()?;
        proj.build()?;

        let mut test_exec = std::process::Command::new( format!("examples/test/.numake/out/{}/test", env!("TARGET")));
        assert_eq!(test_exec.status()?.code(), Some(0));
        Ok(())
    }

    #[test]
    fn mingw_build () -> anyhow::Result<()> {
        let args: NuMakeArgs = NuMakeArgs{
            target: env!("TARGET").to_string().to_string(),
            toolset_compiler: None,
            toolset_linker: None,
            file: "test.numake".to_string(),
            output: "".to_string(),
            workdir: "examples/test".to_string(),
            msvc: None,
            arguments: Some(Vec::from(["mingw".to_string()]))
        };

        let mut proj = Project::new(&args)?;
        proj.setup_lua_vals()?;
        proj.process()?;
        proj.build()?;

        let mut test_exec = std::process::Command::new( "wine");
        test_exec.arg(format!("examples/test/.numake/out/{}/test.exe", env!("TARGET")));

        assert_eq!(test_exec.status()?.code(), Some(0));
        Ok(())
    }
}

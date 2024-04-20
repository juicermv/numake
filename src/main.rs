// @formatter:on

/*
    TODO: Optimization, Refactoring, Error Handling. THIS IS A WIP!
*/

mod numake_file;

mod config;

use std::io;
use std::io::Write;
use clap::Parser;
use std::time::SystemTime;
use crate::config::{Cli, Subcommands};
use crate::numake_file::Project;

fn main() -> anyhow::Result<()> {
    let now = SystemTime::now();

    let cli = Cli::parse();
    match &cli.command {
        Subcommands::Build(args) => {
            let mut proj = Project::new(args);
            proj.setup_lua_vals()?;
            proj.process()?;
            proj.build()?;
            println!(
                "\n\nNuMake done in {}ms!",
                now.elapsed()?.as_millis()
            );
        }

        Subcommands::Inspect(args) => {
            let mut proj = Project::new(args);
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

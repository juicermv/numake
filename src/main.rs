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

fn main() {
    let now = SystemTime::now();

    let cli = Cli::parse();
    match &cli.command {
        Subcommands::Build(args) => {
            let mut proj = Project::new(args);
            proj.setup_lua_vals();
            proj.process();
            proj.build();
            println!(
                "\n\nNuMake done in {}ms!",
                now.elapsed().unwrap().as_millis()
            );
        }

        Subcommands::Inspect(args) => {
            let mut proj = Project::new(args);
            proj.setup_lua_vals();
            proj.process();
            io::stdout()
                .write_all(
                    format!(
                        "{{ \"include\": [ {:?} ] }}",
                        proj.include_paths
                            .iter()
                            .map(|item| { item.to_string() + ", " })
                    )
                    .as_bytes(),
                )
                .unwrap()
        }
    }
}

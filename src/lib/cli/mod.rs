pub mod sub_commands;
pub mod numake_args;
pub mod list_args;

use clap::{
    Args,
    Parser,
    Subcommand,
};
use crate::lib::cli::sub_commands::SubCommands;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli
{
    #[command(subcommand)]
    pub command: SubCommands,
}
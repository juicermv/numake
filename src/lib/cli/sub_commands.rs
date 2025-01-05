use clap::Subcommand;

use crate::lib::cli::list_args::ListArgs;
use crate::lib::cli::numake_args::NuMakeArgs;

#[derive(Subcommand, Clone)]
pub enum SubCommands
{
    /// Run a specific task.
    Build(NuMakeArgs),
    /// List available tasks.
    List(ListArgs),
}
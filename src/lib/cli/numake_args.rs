use clap::{
    Args,
    Parser,
    Subcommand,
};
#[derive(Args, Clone)]
pub struct NuMakeArgs
{
    #[arg(
        help = "Task to run. Use 'numake list' to list available tasks."
    )]
    pub task: String,

    #[arg(long, short, default_value = "project.lua", help = "The script file to read.")]
    pub file: String,

    #[arg(
        long = "working-directory",
        short = 'w',
        default_value = ".",
        help = "Working directory for numake."
    )]
    pub workdir: String,

    #[arg(long, short, help = "Silence numake's output.")]
    pub quiet: bool
}
use clap::{
    Args,
    Parser,
    Subcommand,
};


#[derive(Args, Clone)]
pub struct ListArgs
{
    #[arg(long, short, default_value = "project.lua", help = "The script file to read.")]
    pub file: String,

    #[arg(
        long = "working-directory",
        short = 'w',
        default_value = ".",
        help = "Working directory for numake."
    )]
    pub workdir: String,

    #[arg(long, short, help = "Silence any numake output other than the list itself.")]
    pub quiet: bool
}

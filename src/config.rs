use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Subcommands,
}

#[derive(Subcommand)]
pub enum Subcommands {
    Build(NuMakeArgs),
    Inspect(NuMakeArgs),
}

#[derive(Args)]
pub struct NuMakeArgs {
    #[arg(long, short)]
    pub target: Option<String>,

    #[arg(long="cfg", short='C')]
    pub configuration: Option<String>,

    #[arg(long)]
    pub arch: Option<String>,

    #[arg(long="compiler", short='c')]
    pub toolset_compiler: Option<String>,

    #[arg(long="linker", short='l')]
    pub toolset_linker: Option<String>,

    #[arg(long, short, default_value = "test.numake")]
    pub file: String,

    #[arg(long, short='o', default_value = "out")]
    pub output: String,

    #[arg(long="working-directory", short='w', default_value = ".")]
    pub workdir: String,

    #[arg(long)]
    pub msvc: bool,
}

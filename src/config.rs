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
    #[arg(long)]
    pub target: Option<String>,

    #[arg(long)]
    pub configuration: Option<String>,

    #[arg(long)]
    pub arch: Option<String>,

    #[arg(long)]
    pub toolset_compiler: Option<String>,

    #[arg(long)]
    pub toolset_linker: Option<String>,

    #[arg(long, default_value = "project.numake")]
    pub file: String,

    #[arg(long, default_value = "out")]
    pub output: String,

    #[arg(long, default_value = ".")]
    pub workdir: String,

    #[arg(long)]
    pub msvc: bool,
}

use clap::{
	Args,
	Parser,
	Subcommand,
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli
{
	#[command(subcommand)]
	pub command: Subcommands,
}

#[derive(Subcommand)]
pub enum Subcommands
{
	Build(NuMakeArgs),
	Inspect(InspectArgs),
	List(ListArgs),
}

#[derive(Args)]
pub struct NuMakeArgs
{
	#[arg()]
	pub target: String,

	#[arg(long, short)]
	pub arguments: Option<Vec<String>>,

	#[arg(long = "compiler", short = 'c')]
	pub toolset_compiler: Option<String>,

	#[arg(long = "linker", short = 'l')]
	pub toolset_linker: Option<String>,

	#[arg(long, short, default_value = "project.lua")]
	pub file: String,

	#[arg(long, short = 'o')]
	pub output: Option<String>,

	#[arg(long = "working-directory", short = 'w', default_value = ".")]
	pub workdir: String,

	#[arg(long, short)]
	pub quiet: bool
}

#[derive(Args)]
pub struct InspectArgs
{
	#[arg(long, short)]
	pub arguments: Option<Vec<String>>,

	#[arg(long = "compiler", short = 'c')]
	pub toolset_compiler: Option<String>,

	#[arg(long = "linker", short = 'l')]
	pub toolset_linker: Option<String>,

	#[arg(long, short, default_value = "project.lua")]
	pub file: String,

	#[arg(long, short = 'o')]
	pub output: Option<String>,

	#[arg(long = "working-directory", short = 'w', default_value = ".")]
	pub workdir: String,

	#[arg(long, short)]
	pub quiet: bool
}

#[derive(Args)]
pub struct ListArgs
{
	#[arg(long, short, default_value = "project.lua")]
	pub file: String,

	#[arg(long = "working-directory", short = 'w', default_value = ".")]
	pub workdir: String,

	#[arg(long, short)]
	pub quiet: bool
}

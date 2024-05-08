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
	pub command: SubCommands,
}

#[derive(Subcommand)]
pub enum SubCommands
{
	/// Build a specified target. 
	Build(NuMakeArgs),
	/// Output current workspace in JSON format to stdout. For use with external tools. 
	Inspect(InspectArgs),
	/// List available targets. 
	List(ListArgs),
}

#[derive(Args)]
pub struct NuMakeArgs
{
	#[arg(
		help = "Target to build. Use 'all' or '*' to build all targets. Use 'numake list' to list available targets."
	)]
	pub target: String,

	#[arg(long, short, help = "Arguments to be passed to the script. Pass via 'ARGUMENT=VALUE'.")]
	pub arguments: Option<Vec<String>>,

	#[arg(
		long = "compiler",
		short = 'c',
		help = "Override target's compiler executable. Has no effect on MSVC targets."
	)]
	pub toolset_compiler: Option<String>,

	#[arg(
		long = "linker",
		short = 'l',
		help = "Override target's linker executable. Has no effect on MSVC targets."
	)]
	pub toolset_linker: Option<String>,

	#[arg(long, short, default_value = "project.lua", help = "The script file to read.")]
	pub file: String,

	#[arg(long, short = 'o', help = "Override target's output file name.")]
	pub output: Option<String>,

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

#[derive(Args)]
pub struct InspectArgs
{
	#[arg(long, short, help = "Arguments to be passed to the script. Pass via 'ARGUMENT=VALUE'.")]
	pub arguments: Option<Vec<String>>,

	#[arg(
		long = "compiler",
		short = 'c',
		help = "Override target's compiler executable. Has no effect on MSVC targets."
	)]
	pub toolset_compiler: Option<String>,

	#[arg(
		long = "linker",
		short = 'l',
		help = "Override target's linker executable. Has no effect on MSVC targets."
	)]
	pub toolset_linker: Option<String>,

	#[arg(long, short, default_value = "project.lua", help = "The script file to read.")]
	pub file: String,

	#[arg(
		long = "working-directory",
		short = 'w',
		default_value = ".",
		help = "Working directory for numake."
	)]
	pub workdir: String,

	#[arg(long, short = 'o', help = "Override target's output file name.")]
	pub output: Option<String>,

	#[arg(long, short, help = "Silence any numake output other than the JSON dump.")]
	pub quiet: bool
}

#[derive(Args)]
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

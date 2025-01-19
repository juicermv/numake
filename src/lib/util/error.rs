// SHUT THE FUCK UP THIS IS SNAKE CASE

use thiserror::Error;
#[derive(Debug, Error)]
pub enum NuMakeError {
	#[error("Path exits working directory!")]
	PathOutsideWorkingDirectory,

	#[error("Tried to copy asset to path outside output directory!")]
	AssetCopyPathOutsideWorkingDirectory,

	#[error("No compiler specified/found!")]
	ToolsetCompilerNull,

	#[error("No linker specified/found!")]
	ToolsetLinkerNull,

	#[error("Attempted to add_file with a directory path! Use add_dir instead!")]
	AddFileIsDirectory,

	#[error("Task not found!")]
	TaskNotFound,

	#[error("MSVC target can only be compiled on windows!")]
	MsvcWindowsOnly,

	#[error("Visual C/C++ installation not found! Make sure you have Visual Studio/Build Tools installed!")]
	VcNotFound 
}
// SHUT THE FUCK UP THIS IS SNAKE CASE

#[allow(warnings)]
pub struct NuMakeErrorType<'a>
{
	pub PATH_OUTSIDE_WORKING_DIR: &'a str,
	pub ASSET_COPY_PATH_OUTSIDE_OUTPUT_DIR: &'a str,
	pub TOOLSET_COMPILER_NULL: &'a str,
	pub TOOLSET_LINKER_NULL: &'a str,
	pub ADD_FILE_IS_DIRECTORY: &'a str,
	pub TARGET_NOT_FOUND: &'a str,
	pub MSVC_WINDOWS_ONLY: &'a str,
	pub VC_NOT_FOUND: &'a str
}

pub const NUMAKE_ERROR: NuMakeErrorType<'static> = NuMakeErrorType {
	PATH_OUTSIDE_WORKING_DIR: "Path exits working directory!",
	ASSET_COPY_PATH_OUTSIDE_OUTPUT_DIR: "Tried to copy asset to path outside output directory!",
	TOOLSET_COMPILER_NULL: "No compiler specified/found!",
	TOOLSET_LINKER_NULL: "No linker specified/found!",
	ADD_FILE_IS_DIRECTORY: "Attempted to add_file with a directory path! Use add_dir instead!",
	TARGET_NOT_FOUND: "Target not found!",
	MSVC_WINDOWS_ONLY: "MSVC targets can only be compiled on windows!",
	VC_NOT_FOUND: "Visual C/C++ installation not found! Make sure you have Visual Studio/Build Tools installed!",
};

pub fn to_lua_result<T>(val: anyhow::Result<T>) -> mlua::Result<T>
{
	if val.is_err() {
		Err(mlua::Error::external(val.err().unwrap()))?
	} else {
		Ok(val.ok().unwrap())
	}
}

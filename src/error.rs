// SHUT THE FUCK UP THIS IS SNAKE CASE
pub struct NuMakeErrorType<'a> {
    pub PATH_OUTSIDE_WORKING_DIR: &'a str,
    pub TOOLSET_COMPILER_NULL: &'a str,
    pub TOOLSET_LINKER_NULL: &'a str,
    pub ADD_FILE_IS_DIRECTORY: &'a str,
}

pub const NUMAKE_ERROR: NuMakeErrorType<'static> = NuMakeErrorType {
    PATH_OUTSIDE_WORKING_DIR: "Path exits working directory!",
    TOOLSET_COMPILER_NULL: "No compiler specified/found!",
    TOOLSET_LINKER_NULL: "No linker specified/found!",
    ADD_FILE_IS_DIRECTORY: "Attempted to add_file with a directory path! Use add_dir instead!",
};

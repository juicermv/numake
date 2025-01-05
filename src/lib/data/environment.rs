use std::path::PathBuf;

#[derive(Clone, Default)]
pub struct Environment {
    pub numake_directory: PathBuf,
    pub project_directory: PathBuf,
    pub project_file: PathBuf,
}
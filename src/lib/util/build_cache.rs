use crate::lib::data::environment::Environment;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Default, Debug, Clone)]
pub struct BuildCache {
	directory: PathBuf,
}

impl BuildCache {
	pub fn new(environment: Environment) -> anyhow::Result<Self> {
		let directory = environment.numake_directory.join(".cache");
		if !directory.exists() {
			fs::create_dir_all(&directory)?;
		}

		Ok(BuildCache { directory })
	}

	pub fn write_set(
		&self,
		array_name: &str,
		contents: HashSet<String>,
	) -> anyhow::Result<()> {
		let file_path = self.directory.join(array_name.to_string() + ".bcache");
		let bytes = bitcode::encode(&contents);
		fs::write(&file_path, bytes)?;
		Ok(())
	}

	pub fn read_set(
		&self,
		array_name: &str,
	) -> anyhow::Result<HashSet<String>> {
		let file_path = self.directory.join(array_name.to_string() + ".bcache");
        if !file_path.exists() {
            return Ok(HashSet::default());
        }
        
		match bitcode::decode::<HashSet<String>>(
			fs::read(file_path)?.as_slice(),
		) {
            Ok(v) => Ok(v),
            Err(_) => Ok(HashSet::default())
        }
	}
}
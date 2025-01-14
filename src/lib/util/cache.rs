use crate::lib::data::environment::Environment;
use crate::lib::util::hash_string;
use bzip2::read::{BzDecoder, BzEncoder};
use bzip2::Compression;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::{fs, io, path::PathBuf, str::FromStr};
use toml::Table;

#[derive(Default)]
pub struct Cache {
	user_values: Arc<Mutex<Table>>,

	toml: Arc<Mutex<Table>>,
	directory: PathBuf,
}

impl Cache {
	pub fn new(environment: Environment) -> anyhow::Result<Self> {
		let directory = environment.numake_directory.join(".cache");
		if !directory.exists() {
			fs::create_dir_all(&directory)?;
		}

		let toml_path = directory.join("cache.toml");
		if !toml_path.exists() {
			fs::write(&toml_path, "")?;
		}

		let table = Table::from_str(&fs::read_to_string(&toml_path)?)?;
		let mut user_values = Table::new();

		if table.contains_key("workspace") {
			user_values =
				table.get("workspace").unwrap().as_table().unwrap().clone();
		}

		Ok(Cache {
			toml: Arc::new(Mutex::new(table)),
			user_values: Arc::new(Mutex::new(user_values)),
			directory,
		})
	}

	pub fn write_file<T: AsRef<[u8]>>(
		&mut self,
		name: &String,
		data: T,
	) -> anyhow::Result<PathBuf> {
		let file_path = self.directory.join(hash_string(name) + ".file");
		let mut compressor = BzEncoder::new(data.as_ref(), Compression::best());
		io::copy(&mut compressor, &mut File::create(file_path.clone())?)?;
		Ok(file_path)
	}

	pub fn read_file(
		&mut self,
		name: &String,
	) -> anyhow::Result<Vec<u8>> {
		let file_path = self.directory.join(hash_string(name) + ".file");
		let mut buf = Vec::new();
		let mut decoder = BzDecoder::new(File::open(file_path)?);
		io::copy(&mut decoder, &mut buf)?;

		Ok(buf)
	}

	pub fn check_file_exists(
		&mut self,
		name: &String,
	) -> bool {
		self.directory.join(hash_string(name) + ".file").exists()
	}

	pub fn get_dir(
		&mut self,
		name: &String,
	) -> anyhow::Result<PathBuf> {
		let dir = self.directory.join(hash_string(name));
		if !dir.exists() {
			fs::create_dir_all(&dir)?;
		}

		Ok(dir)
	}

	pub fn check_dir_exists(
		&mut self,
		name: &String,
	) -> bool {
		self.directory.join(hash_string(name)).exists()
	}

	pub fn set_value(
		&mut self,
		key: &String,
		val: toml::Value,
	) -> anyhow::Result<()> {
		(*self.toml.lock().unwrap()).insert(hash_string(key), val);
		Ok(())
	}

	pub fn get_value(
		&mut self,
		key: &String,
	) -> Option<toml::Value> {
		match (*self.toml.lock().unwrap()).get(&hash_string(key)) {
			Some(v) => Some(v.clone()),
			None => None,
		}
	}

	pub fn pop_value(
		&mut self,
		key: &String,
	) {
		(*self.toml.lock().unwrap()).remove(&hash_string(key));
	}

	pub fn check_key_exists(
		&mut self,
		key: &String,
	) -> bool {
		(*self.toml.lock().unwrap()).contains_key(&hash_string(key))
	}

	pub fn set_user_value(
		&mut self,
		key: &String,
		val: toml::Value,
	) -> anyhow::Result<()> {
		(*self.user_values.lock().unwrap()).insert(hash_string(key), val);
		Ok(())
	}

	pub fn get_user_value(
		&mut self,
		key: &String,
	) -> Option<toml::Value> {
		match (*self.user_values.lock().unwrap()).get(&hash_string(key)) {
			Some(v) => Some(v.clone()),
			None => None,
		}
	}

	pub fn pop_user_value(
		&mut self,
		key: &String,
	) {
		(*self.user_values.lock().unwrap()).remove(&hash_string(key));
	}

	pub fn flush(&mut self) -> io::Result<()> {
		(*self.toml.lock().unwrap()).insert(
			"workspace".to_string(),
			toml::Value::Table((*self.user_values.lock().unwrap()).clone()),
		);
		fs::write(self.directory.join("cache.toml"), (*self.toml.lock().unwrap()).to_string())
	}
}

impl Clone for Cache {
	fn clone(&self) -> Self {
		Cache {
			user_values: Arc::clone(&self.user_values),
			toml: Arc::clone(&self.toml),
			directory: self.directory.clone(),
		}
	}
}

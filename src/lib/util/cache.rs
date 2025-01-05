use std::{
	fs,
	io,
	path::PathBuf,
	str::FromStr,
};
use bzip2::{Action, Compress, Compression, Decompress};
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use toml::Table;
use crate::lib::data::environment::Environment;
use crate::lib::util::hash_string;

#[derive(Clone, Default)]
pub struct Cache
{
	pub user_values: Table,

	toml: Table,
	directory: PathBuf,
}

impl Cache
{
	pub fn new(environment: Environment) -> anyhow::Result<Self>
	{
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
			toml: table,
			user_values,
			directory,
		})
	}

	pub fn write_file<T: AsRef<[u8]>>(
		&mut self,
		name: &String,
		data: T,
	) -> anyhow::Result<PathBuf>
	{
		let file_path = self.directory.join(hash_string(name) + ".file");
		let mut compressor = Compress::new(Compression::best(), 30);
		let mut compressed_data:Vec<u8> = Vec::new();
		compressor.compress(data, &mut compressed_data, Action::Run)?;
		fs::write(&file_path, compressed_data)?;

		Ok(file_path)
	}

	pub fn read_file(
		&mut self,
		name: &String,
	) -> anyhow::Result<Vec<u8>>
	{
		let file_path = self.directory.join(hash_string(name) + ".file");
		let buffer = fs::read(file_path)?;
		let mut decompressor = Decompress::new(false);
		let mut output: Vec<u8> = Vec::new();
		decompressor.decompress_vec(&buffer, &mut output)?;
		Ok(output)
	}

	pub fn check_file_exists(
		&mut self,
		name: &String,
	) -> bool
	{
		self.directory.join(hash_string(name) + ".file").exists()
	}

	pub fn get_dir(
		&mut self,
		name: &String,
	) -> anyhow::Result<PathBuf>
	{
		let dir = self.directory.join(hash_string(name));
		if !dir.exists() {
			fs::create_dir_all(&dir)?;
		}

		Ok(dir)
	}

	pub fn check_dir_exists(
		&mut self,
		name: &String,
	) -> bool
	{
		self.directory.join(hash_string(name)).exists()
	}

	pub fn set_value(
		&mut self,
		key: &String,
		val: toml::Value,
	) -> anyhow::Result<()>
	{
		self.toml.insert(hash_string(key), val);
		Ok(())
	}

	pub fn get_value(
		&mut self,
		key: &String,
	) -> Option<&toml::Value>
	{
		self.toml.get(&hash_string(key))
	}

	pub fn pop_value(
		&mut self,
		key: &String,
	)
	{
		self.toml.remove(&hash_string(key));
	}

	pub fn check_key_exists(
		&mut self,
		key: &String,
	) -> bool
	{
		self.toml.contains_key(&hash_string(key))
	}

	pub fn set_user_value(
		&mut self,
		key: &String,
		val: toml::Value,
	) -> anyhow::Result<()>
	{
		self.user_values.insert(hash_string(key), val);
		Ok(())
	}

	pub fn get_user_value(
		&mut self,
		key: &String,
	) -> Option<&toml::Value>
	{
		self.user_values.get(&hash_string(key))
	}

	pub fn pop_user_value(
		&mut self,
		key: &String,
	)
	{
		self.user_values.remove(&hash_string(key));
	}

	pub fn flush(&mut self) -> io::Result<()>
	{
		self.toml.insert(
			"workspace".to_string(),
			toml::Value::Table(self.user_values.clone()),
		);
		fs::write(self.directory.join("cache.toml"), self.toml.to_string())
	}
}

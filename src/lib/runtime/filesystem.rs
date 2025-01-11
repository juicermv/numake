use crate::lib::data::environment::Environment;
use mlua::{UserData, UserDataMethods};
use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Filesystem {
	environment: Environment,
}

impl Filesystem {
	pub fn new(environment: Environment) -> Self {
		Filesystem { environment }
	}

	pub fn walk_dir(
		&self,
		path_buf: PathBuf,
		recursive: bool,
	) -> anyhow::Result<Vec<PathBuf>> {
		let mut path_vec: Vec<PathBuf> = Vec::new();

		for entry in fs::read_dir(path_buf)? {
			let path = dunce::canonicalize(entry?.path())?;
			if path.is_dir() && recursive {
				path_vec.append(&mut self.walk_dir(path.clone(), true)?.clone())
			}
			if path.is_file() {
				path_vec.push(path.clone());
			}
		}

		Ok(path_vec)
	}
}

impl UserData for Filesystem {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut(
			"walk",
			|_, this, (path, recursive): (String, bool)| match this
				.walk_dir(dunce::canonicalize(path)?, recursive)
			{
				Ok(paths) => {
					let ret: Vec<String> = paths
						.into_iter()
						.map(|path_buf: PathBuf| {
							let buf = path_buf.clone();
							let buf_str = buf.display().to_string();
							buf_str
						})
						.collect();
					Ok(ret)
				}

				Err(e) => Err(mlua::Error::external(e)),
			},
		)
	}
}

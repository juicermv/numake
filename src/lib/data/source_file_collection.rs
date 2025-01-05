use crate::lib::data::source_file_type::SourceFileType;
use mlua::IntoLua;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::PathBuf;
use strum::IntoEnumIterator;

#[derive(Clone, Debug, Default)]
pub struct SourceFileCollection {
	files: HashMap<SourceFileType, HashSet<PathBuf>>,
}

impl SourceFileCollection {
	pub fn new() -> Self {
		let mut files: HashMap<SourceFileType, HashSet<PathBuf>> = HashMap::new();
		for source_file_type in SourceFileType::iter() {
			files.insert(source_file_type, HashSet::new());
		}

		Self { files }
	}

	pub fn insert(
		&mut self,
		file: impl Into<PathBuf>,
	) {
		let pathbuf = file.into();
		let file_type = SourceFileType::from(pathbuf.clone());
		match self.files.get(&file_type.clone()) {
			Some(files) => {
				let mut files_mut = files.clone();
				files_mut.insert(pathbuf);
				self.files.insert(file_type, files_mut);
			}

			None => {
				let mut files_mut = HashSet::new();
				files_mut.insert(pathbuf);
				self.files.insert(file_type, files_mut);
			}
		}
	}

	pub fn get(
		&self,
		file_type: &SourceFileType,
	) -> Vec<PathBuf> {
		match self.files.get(file_type) {
			Some(files) => {
				files.into_iter().map(
					|file| file.clone()
				).collect()
			}

			None => {
				Vec::new()
			}
		}
	}
}

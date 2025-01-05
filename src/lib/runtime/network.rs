use crate::lib::util::cache::Cache;
use crate::lib::data::environment::Environment;
use mlua::{UserData, UserDataMethods};
use serde::Serialize;
use std::io::Cursor;
use zip::ZipArchive;
use crate::lib::util::ui::NumakeUI;

#[derive(Clone, Serialize)]
pub struct Network {
	#[serde(skip)]
	environment: Environment,
	#[serde(skip)]
	ui: NumakeUI,
	#[serde(skip)]
	cache: Cache,
}

impl Network {
	pub fn new(
		environment: Environment,
		ui: NumakeUI,
		cache: Cache,
	) -> Network {
		Network {
			environment,
			ui,
			cache,
		}
	}
	pub unsafe fn download_zip(
		&mut self,
		url: String,
	) -> anyhow::Result<String> {
		if self.cache.check_dir_exists(&url) {
			self.ui
				.progress_manager
				.println(self.ui.format_ok("Archive contents found on disk.".to_string()))?;
			Ok(self
				.cache
				.get_dir(&url)?
				.to_str()
				.unwrap_or("ERROR")
				.to_string())
		} else {
			if self.cache.check_file_exists(&url) {
				let spinner = self.ui.spinner(self.ui.format_info(format!(
					"Archive found in cache. Extracting... [{}]",
					&url
				)));
				ZipArchive::new(Cursor::new(self.cache.read_file(&url)?))?
					.extract(self.cache.get_dir(&url)?)?;
				self.ui
					.progress_manager
					.println(self.ui.format_ok("Done!".to_string()))?;
				spinner.finish();

				Ok(self.cache.get_dir(&url)?.to_str().unwrap_or("ERROR").to_string())
			} else {
				let response = reqwest::blocking::get(&url)?;
				let status = response.status();
				if status.is_success() {
					let spinner = (self.ui).spinner(
						"Downloading & extracting archive...".to_string(),
					);
					(self.ui).progress_manager.println((self.ui).format_ok(
						format!(
							"Server responded with {}! [{}]",
							response.status(),
							&url
						),
					))?;
					let path = self.cache.get_dir(&url)?;

					let data = response.bytes()?;
					self.cache.write_file(&url, data.clone())?;
					ZipArchive::new(Cursor::new(data.clone()))?
						.extract(&path)?;
					spinner.finish_and_clear();
					(self.ui).progress_manager.println(
						(self.ui)
							.format_ok(format!("Done extracting! [{}]", url)),
					)?;

					Ok(path.to_str().unwrap().to_string())
				} else {
					anyhow::bail!("Server responded with {}! [{}]", status, url)
				}
			}
		}
	}
}

impl UserData for Network {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut("zip", |lua, this, url| unsafe {
			match this.download_zip(url) {
				Ok(path) => Ok(path),
				Err(e) => Err(mlua::Error::RuntimeError(format!("{:?}", e))),
			}
		})
	}
}

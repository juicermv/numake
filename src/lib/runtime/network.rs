use crate::lib::data::environment::Environment;
use crate::lib::ui::format::info::Info;
use crate::lib::ui::format::ok;
use crate::lib::ui::UI;
use crate::lib::util::cache::Cache;
use mlua::{UserData, UserDataMethods};
use std::io::Cursor;
use zip::ZipArchive;

#[derive(Clone)]
pub struct Network {
	environment: Environment,
	ui: UI,
	cache: Cache,
}

impl Network {
	pub fn new(
		environment: Environment,
		ui: UI,
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
				.println("Archive contents found on disk.", Info::default());
			Ok(self
				.cache
				.get_dir(&url)?
				.to_str()
				.unwrap_or("ERROR")
				.to_string())
		} else {
			if self.cache.check_file_exists(&url) {
				let spinner = self.ui.create_spinner(format!(
					"Archive found in cache. Extracting... [{}]",
					&url
				));
				ZipArchive::new(Cursor::new(self.cache.read_file(&url)?))?
					.extract(self.cache.get_dir(&url)?)?;
				self.ui.println("Done!", ok::Ok::default());
				spinner.finish();

				Ok(self
					.cache
					.get_dir(&url)?
					.to_str()
					.unwrap_or("ERROR")
					.to_string())
			} else {
				let response = reqwest::blocking::get(&url)?;
				let status = response.status();
				if status.is_success() {
					let spinner = self
						.ui
						.create_spinner("Downloading & extracting archive...");
					self.ui.println(
						format!(
							"Server responded with {}! [{}]",
							response.status(),
							&url
						),
						ok::Ok::default(),
					);
					let path = self.cache.get_dir(&url)?;

					let data = response.bytes()?;
					self.cache.write_file(&url, data.clone())?;
					ZipArchive::new(Cursor::new(data.clone()))?
						.extract(&path)?;
					spinner.finish_and_clear();
					self.ui.println(
						format!("Done extracting! [{}]", url),
						ok::Ok::default(),
					);

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

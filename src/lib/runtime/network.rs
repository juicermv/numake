use crate::lib::data::environment::Environment;
use crate::lib::ui::NumakeUI;
use crate::lib::util::hash_string;
use mlua::{IntoLua, UserData, UserDataMethods};
use std::io::Cursor;
use serde::Serialize;
use zip::ZipArchive;


#[derive(Clone, Serialize)]
pub struct Network {
	#[serde(skip)]
	environment:  Environment,
	#[serde(skip)]
	ui:  NumakeUI,
}

impl Network {

    pub fn new(environment:  Environment, ui:  NumakeUI) -> Network {
        Network { environment, ui }
    }
	pub unsafe fn download_zip(
		&mut self,
		url: String,
	) -> anyhow::Result<String> {
		let response = reqwest::blocking::get(&url)?;
		let status = response.status();
		if status.is_success() {
			let spinner = (self.ui)
				.spinner("Downloading & extracting archive...".to_string());
			(self.ui).progress_manager.println((self.ui).format_ok(format!(
				"Server responded with {}! [{}]",
				response.status(),
				&url
			)))?;
			let path =
				(self.environment).numake_directory.join(hash_string(&url));

			ZipArchive::new(Cursor::new(response.bytes()?))?.extract(&path)?;
			spinner.finish_and_clear();
			(self.ui).progress_manager.println(
				(self.ui).format_ok(format!("Done extracting! [{}]", url)),
			)?;

			Ok(path.to_str().unwrap().to_string())
		} else {
			anyhow::bail!("Server responded with {}! [{}]", status, url)
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

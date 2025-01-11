use crate::lib::runtime::storage::Storage;
use crate::lib::ui;
use crate::lib::ui::format::info::Info;
use crate::lib::ui::format::{error, ok};
use crate::lib::ui::{format, UI};
use anyhow::anyhow;
use mlua::prelude::LuaValue;
use mlua::{FromLua, Lua, Value};
use std::process::{Command, ExitStatus};

#[derive(Clone)]
pub struct System {
	ui: UI,
}

impl System {
	pub fn new(ui: UI) -> System {
		System { ui }
	}

	pub(crate) fn execute(
		&self,
		cmd: &mut Command,
	) -> mlua::Result<i32> {
		let result = cmd.output();
		match result {
			Err(err) => Err(mlua::Error::runtime(format!(
				"Error trying to execute {}! {}",
				cmd.get_program().to_str().unwrap(),
				err
			))),

			Ok(output) => {
				let stderr =
					String::from_utf8_lossy(&output.stderr).to_string();

				if output.status.success() {
					if !stderr.is_empty() {
						self.ui.println(stderr.clone() + "\n", Info::default());
					}

					self.ui.println(
						format!(
							"{} exited with {}",
							cmd.get_program().to_str().unwrap(),
							output.status
						),
						ok::Ok::default(),
					);
					Ok(output.status.code().unwrap_or(0))
				} else {
					self.ui.println(
						format!(
							"{} exited with {}",
							cmd.get_program().to_str().unwrap(),
							output.status
						),
						error::Error::default(),
					);
					Err(mlua::Error::runtime(stderr))
				}
			}
		}
	}

	pub(crate) fn msvc_execute(
		&self,
		cmd: &mut Command,
	) -> anyhow::Result<ExitStatus> {
		let result = cmd.output();

		match result {
			Err(err) => Err(anyhow!(format!(
				"Error trying to execute {}! {}",
				cmd.get_program().to_str().unwrap(),
				err
			))),

			Ok(output) => {
				let stdout =
					String::from_utf8_lossy(&output.stdout).to_string();

				if output.status.success() {
					self.ui.println(if stdout.contains(": warning ") {
						stdout.clone()
					} else {
						stdout.clone()
					} + "\n", Info::default());

					self.ui.println(format!(
						"{} exited with {}",
						cmd.get_program().to_str().unwrap(),
						output.status
					), ok::Ok::default());
					Ok(output.status)
				} else {
					self.ui.println(format!(
						"{} exited with {}",
						cmd.get_program().to_str().unwrap(),
						output.status
					), error::Error::default());
					Err(anyhow!(stdout))
				}
			}
		}
	}
}

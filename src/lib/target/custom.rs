use std::process::{
	Command,
	ExitStatus,
};

use mlua::{
	FromLua,
	Lua,
	prelude::LuaValue,
	UserData,
	UserDataFields,
	Value,
};
use serde::Serialize;
use crate::lib::target::{Target, TargetTrait, VSCodeProperties};
use crate::lib::workspace::LuaWorkspace;

#[derive(Clone, Serialize)]
pub struct CustomTarget
{
	pub sub_targets: Vec<Target>,
	pub name: String,
	pub description: String,
}

impl CustomTarget
{
	pub fn new(
		name: String,
		description: String,
	) -> Self
	{
		CustomTarget {
			name,
			description,
			sub_targets: Vec::new(),
		}
	}
}

impl TargetTrait for CustomTarget
{
	fn build(
		&self,
		parent_workspace: &mut LuaWorkspace,
	) -> anyhow::Result<()>
	{
		for target in self.sub_targets.clone() {
			target.build(parent_workspace)?;
		}
		Ok(())
	}

	fn execute(
		&self,
		_: &mut Command,
	) -> anyhow::Result<ExitStatus>
	{
		Ok(ExitStatus::default())
	}

	fn set_vscode_props(&mut self, lua_workspace: &mut LuaWorkspace) -> anyhow::Result<VSCodeProperties>
	{
		Ok(VSCodeProperties {
			compiler_path: "custom".to_string(),
			default_includes: Vec::default(),
			intellisense_mode: "".to_string(),
		})
	}
}

impl UserData for CustomTarget
{
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F)
	{
		fields.add_field_method_set(
			"sub_targets",
			|_, this, targets: Vec<Target>| {
				this.sub_targets = targets;
				Ok(())
			},
		);
	}
}

impl FromLua for CustomTarget
{
	fn from_lua(
		value: LuaValue,
		_: &Lua,
	) -> mlua::Result<Self>
	{
		match value {
			Value::UserData(user_data) => {
				if user_data.is::<Self>() {
					Ok(user_data.borrow::<Self>()?.clone())
				} else {
					Err(mlua::Error::UserDataTypeMismatch)
				}
			}

			_ => Err(mlua::Error::UserDataTypeMismatch),
		}
	}
}

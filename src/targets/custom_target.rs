use std::process::{
	Command,
	ExitStatus,
};

use indicatif::ProgressBar;
use mlua::{
	prelude::{
		LuaFunction,
		LuaValue,
	},
	FromLua,
	Function,
	Lua,
	UserData,
	UserDataFields,
	Value,
};
use serde::Serialize;

use crate::{
	targets::target::{
		Target,
		TargetTrait,
	},
	workspace::LuaWorkspace,
};

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
		progress: &ProgressBar,
	) -> anyhow::Result<()>
	{
		for target in self.sub_targets.clone() {
			target.build(parent_workspace, progress)?;
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
}

impl UserData for CustomTarget
{
	fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F)
	{
		fields.add_field_method_set(
			"sub_targets",
			|lua, this, targets: Vec<Target>| {
				this.sub_targets = targets;
				Ok(())
			},
		);
	}
}

impl<'lua> FromLua<'lua> for CustomTarget
{
	fn from_lua(
		value: LuaValue<'lua>,
		_: &'lua Lua,
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
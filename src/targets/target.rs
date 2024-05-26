use std::process::{
	Command,
	ExitStatus,
};

use mlua::{
	FromLua,
	IntoLua,
	Lua,
	Value,
};
use serde::{
	Serialize,
	Serializer,
};

use crate::{
	targets::{
		custom_target::CustomTarget,
		generic_target::GenericTarget,
		mingw_target::MinGWTarget,
		msvc_target::MSVCTarget,
	},
	workspace::LuaWorkspace,
};

#[derive(Serialize, Clone, Default)]
pub struct VSCodeProperties
{
	pub compiler_path: String,
	pub default_includes: Vec<String>,
	pub intellisense_mode: String,
}

pub trait TargetTrait
{
	fn build(
		&self,
		parent_workspace: &mut LuaWorkspace,
	) -> anyhow::Result<()>;

	fn execute(
		&self,
		cmd: &mut Command,
	) -> anyhow::Result<ExitStatus>;

	fn set_vscode_props(&mut self, lua_workspace: &mut LuaWorkspace) -> anyhow::Result<VSCodeProperties>;
}

#[derive(Clone)]
pub enum Target
{
	Generic(GenericTarget),
	MSVC(MSVCTarget),
	MinGW(MinGWTarget),
	Custom(CustomTarget),
}

impl Target
{
	pub fn get_name(&self) -> String
	{
		match self {
			Target::Generic(target) => target.name.clone(),
			Target::MSVC(target) => target.name.clone(),
			Target::MinGW(target) => target.name.clone(),
			Target::Custom(target) => target.name.clone(),
		}
	}
}

impl Serialize for Target
{
	fn serialize<S>(
		&self,
		serializer: S,
	) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match self {
			Target::MinGW(target) => target.serialize(serializer),
			Target::MSVC(target) => target.serialize(serializer),
			Target::Generic(target) => target.serialize(serializer),
			Target::Custom(target) => target.serialize(serializer),
		}
	}
}

impl TargetTrait for Target
{
	fn build(
		&self,
		parent_workspace: &mut LuaWorkspace,
	) -> anyhow::Result<()>
	{
		match self {
			Target::Generic(target) => target.build(parent_workspace),
			Target::MSVC(target) => target.build(parent_workspace),
			Target::MinGW(target) => target.build(parent_workspace),
			Target::Custom(target) => target.build(parent_workspace),
		}
	}

	fn execute(
		&self,
		cmd: &mut Command,
	) -> anyhow::Result<ExitStatus>
	{
		match self {
			Target::Generic(target) => target.execute(cmd),
			Target::MSVC(target) => target.execute(cmd),
			Target::MinGW(target) => target.execute(cmd),
			Target::Custom(target) => target.execute(cmd),
		}
	}

	fn set_vscode_props(&mut self, lua_workspace: &mut LuaWorkspace) -> anyhow::Result<VSCodeProperties>
	{
		match self {
			Target::Generic(target) => target.set_vscode_props(lua_workspace),
			Target::MSVC(target) => target.set_vscode_props(lua_workspace),
			Target::MinGW(target) => target.set_vscode_props(lua_workspace),
			Target::Custom(target) => target.set_vscode_props(lua_workspace),
		}
	}
}

impl<'lua> IntoLua<'lua> for Target
{
	fn into_lua(
		self,
		lua: &'lua Lua,
	) -> mlua::Result<Value<'lua>>
	{
		match self {
			Target::Generic(target) => target.into_lua(lua),
			Target::MSVC(target) => target.into_lua(lua),
			Target::MinGW(target) => target.into_lua(lua),
			Target::Custom(target) => target.into_lua(lua),
		}
	}
}

impl<'lua> FromLua<'lua> for Target
{
	fn from_lua(
		value: Value<'lua>,
		_: &'lua Lua,
	) -> mlua::Result<Self>
	{
		match value {
			Value::UserData(user_data) => {
				if user_data.is::<MSVCTarget>() {
					Ok(Self::MSVC(user_data.borrow::<MSVCTarget>()?.clone()))
				} else if user_data.is::<GenericTarget>() {
					Ok(Self::Generic(
						user_data.borrow::<GenericTarget>()?.clone(),
					))
				} else if user_data.is::<MinGWTarget>() {
					Ok(Self::MinGW(user_data.borrow::<MinGWTarget>()?.clone()))
				} else if user_data.is::<CustomTarget>() {
					Ok(Self::Custom(
						user_data.borrow::<CustomTarget>()?.clone(),
					))
				} else {
					Err(mlua::Error::UserDataTypeMismatch)
				}
			}

			_ => Err(mlua::Error::UserDataTypeMismatch),
		}
	}
}

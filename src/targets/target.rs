use std::process::{
	Command,
	ExitStatus,
};

use indicatif::ProgressBar;
use mlua::{
	FromLua,
	IntoLua,
	Lua,
	Value,
};
use serde::Serialize;

use crate::{
	targets::{
		generic_target::GenericTarget,
		mingw_target::MINGWTarget,
		msvc_target::MSVCTarget,
	},
	workspace::LuaWorkspace,
};

pub trait TargetTrait
{
	fn build(
		&self,
		parent_workspace: &mut LuaWorkspace,
		progress: &ProgressBar,
	) -> anyhow::Result<()>;

	fn execute(
		&self,
		cmd: &mut Command,
	) -> anyhow::Result<ExitStatus>;
}

#[derive(Clone, Serialize)]
pub enum Target
{
	Generic(GenericTarget),
	MSVC(MSVCTarget),
	MINGW(MINGWTarget),
}

impl Target
{
	pub fn get_name(&self) -> String
	{
		match self {
			Target::Generic(target) => target.name.clone(),
			Target::MSVC(target) => target.name.clone(),
			Target::MINGW(target) => target.name.clone(),
		}
	}
}

impl TargetTrait for Target
{
	fn build(
		&self,
		parent_workspace: &mut LuaWorkspace,
		progress: &ProgressBar,
	) -> anyhow::Result<()>
	{
		match self {
			Target::Generic(target) => target.build(parent_workspace, progress),
			Target::MSVC(target) => target.build(parent_workspace, progress),
			Target::MINGW(target) => target.build(parent_workspace, progress),
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
			Target::MINGW(target) => target.execute(cmd),
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
			Target::MINGW(target) => target.into_lua(lua),
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
				} else if user_data.is::<MINGWTarget>() {
					Ok(Self::MINGW(user_data.borrow::<MINGWTarget>()?.clone()))
				} else {
					Err(mlua::Error::runtime(
						"Tried to convert invalid target type to Target!",
					))
				}
			}

			_ => unreachable!(),
		}
	}
}

use crate::lib::data::project_language::ProjectLanguage;
use mlua::prelude::{LuaResult, LuaValue};
use mlua::Error::UserDataTypeMismatch;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::{EnumString, IntoStaticStr};

#[derive(
	Debug, Clone, IntoStaticStr, Default, Serialize, Deserialize, EnumString,
)]
pub enum ProjectType {
	#[default]
	Executable,
	StaticLibrary,
	DynamicLibrary,
}

impl FromLua for ProjectType {
	fn from_lua(
		value: Value,
		lua: &Lua,
	) -> mlua::Result<Self> {
		match value {
			Value::String(str) => Self::from_str(&str.to_str()?.to_string())
				.map_err(|e| e.into_lua_err()),
			_ => Err(mlua::Error::UserDataTypeMismatch),
		}
	}
}

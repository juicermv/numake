use crate::lib::data::flag_type::FlagType;
use mlua::prelude::{LuaResult, LuaValue};
use mlua::Error::UserDataTypeMismatch;
use mlua::{ExternalError, FromLua, IntoLua, Lua, Value};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::{EnumString, IntoStaticStr};

#[derive(
	Debug, Clone, IntoStaticStr, Default, Serialize, Deserialize, EnumString,
)]
pub enum ProjectLanguage {
	#[default]
	C,
	CPP,
}

impl FromLua for ProjectLanguage {
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

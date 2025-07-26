use std::str::FromStr;
use mlua::{ExternalError, FromLua, Lua, Value};
use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, IntoStaticStr};

#[derive(
    Debug,
    Clone,
    IntoStaticStr,
    Default,
    Serialize,
    Deserialize,
    EnumString,
    Eq,
    PartialEq,
)]
pub enum FlagType {
    #[default]
    Compiler,
    Linker,
    RC,
    WINDRES,
}

impl FromLua for FlagType {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match value {
            Value::String(str) => Self::from_str(&str.to_str()?.to_string()).map_err(|e| e.into_lua_err()),
            _ => Err(mlua::Error::UserDataTypeMismatch)
        }
    }
}

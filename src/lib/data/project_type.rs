use mlua::{FromLua, IntoLua, Lua};
use mlua::Error::UserDataTypeMismatch;
use mlua::prelude::{LuaResult, LuaValue};
use mlua::Value::String;
use strum_macros::IntoStaticStr;
use crate::lib::data::source_file_type::SourceFileType;


#[derive(Debug, Clone, IntoStaticStr, Default)]
pub enum ProjectType {
    #[default]
    Executable,
    StaticLibrary,
    DynamicLibrary,
}

impl IntoLua for ProjectType {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let me_str: &str = self.into();
        me_str.into_lua(lua)
    }
}

impl FromLua for ProjectType {
    fn from_lua(lua_value: LuaValue, lua: &Lua) -> LuaResult<Self> {
        match lua_value {
            LuaValue::String(ref string) => {
                let project_type_str = string.to_str()?;
                match project_type_str.to_lowercase().as_str() {
                    "executable" => Ok(ProjectType::Executable),
                    "staticlibrary" => Ok(ProjectType::StaticLibrary),
                    "dynamiclibrary" => Ok(ProjectType::DynamicLibrary),
                    _ => Err(UserDataTypeMismatch)
                }
            }

            LuaValue::Number(ref number) => {
                match (number - 0f64) as i64 {
                    1 => Ok(ProjectType::Executable),
                    2 => Ok(ProjectType::StaticLibrary),
                    3 => Ok(ProjectType::DynamicLibrary),
                    _ => Err(UserDataTypeMismatch)
                }
            }

            _ => Err(UserDataTypeMismatch)
        }
    }
}
use crate::lib::data::project_type::ProjectType;
use mlua::prelude::{LuaResult, LuaValue};
use mlua::Error::UserDataTypeMismatch;
use mlua::{FromLua, IntoLua, Lua};
use strum_macros::IntoStaticStr;

#[derive(Debug, Clone, IntoStaticStr, Default)]
pub enum ProjectLanguage {
	#[default]
	C,
	CPP,
}

impl IntoLua for ProjectLanguage {
	fn into_lua(
		self,
		lua: &Lua,
	) -> LuaResult<LuaValue> {
		let me_str: &str = self.into();
		me_str.into_lua(lua)
	}
}

impl FromLua for ProjectLanguage {
	fn from_lua(
		lua_value: LuaValue,
		lua: &Lua,
	) -> LuaResult<Self> {
		match lua_value {
			LuaValue::String(ref string) => {
				let project_type_str = string.to_str()?;
				match project_type_str.to_lowercase().as_str() {
					"c" => Ok(ProjectLanguage::C),
					"cpp" => Ok(ProjectLanguage::CPP),
					"c++" => Ok(ProjectLanguage::CPP),
					_ => Err(UserDataTypeMismatch),
				}
			}

			LuaValue::Number(ref number) => match (number - 0f64) as i64 {
				1 => Ok(ProjectLanguage::C),
				2 => Ok(ProjectLanguage::CPP),
				_ => Err(UserDataTypeMismatch),
			},

			_ => Err(UserDataTypeMismatch),
		}
	}
}

use std::path::PathBuf;
use mlua::{IntoLua, Lua};
use mlua::prelude::{LuaResult, LuaString, LuaValue};
use strum_macros::{EnumIter, IntoStaticStr};

#[derive(Debug, Copy, Clone, EnumIter, Hash, PartialEq, Eq, IntoStaticStr)]
pub enum SourceFileType {
    Code,
    Resource,
    ModuleDefinition,
    Unknown,
}


impl From<&PathBuf> for SourceFileType {
    fn from(path: &PathBuf) -> Self {
        match path.extension() {
            None => SourceFileType::Unknown,
            Some(extension) => {
                match extension.to_str() {
                    None => SourceFileType::Unknown,
                    Some(extension_str) => {
                        match extension_str {
                            "cpp" => SourceFileType::Code,
                            "c" => SourceFileType::Code,
                            "cxx" => SourceFileType::Code,
                            "rc" => SourceFileType::Resource,
                            "def" => SourceFileType::ModuleDefinition,
                            _ => SourceFileType::Unknown,
                        }
                    }
                }
            }
        }
    }
}

impl SourceFileType {
    pub fn from(path: impl Into<PathBuf>) -> SourceFileType {
        Self::from(path.into())
    }
}

impl IntoLua for SourceFileType {
    fn into_lua(self, lua: &Lua) -> LuaResult<LuaValue> {
        let me_str: &str = self.into();
        me_str.into_lua(lua)
    }
}
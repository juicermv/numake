use std::path::PathBuf;
use mlua::{IntoLua, Lua};
use mlua::prelude::{LuaResult, LuaValue};
use serde::{Deserialize, Serialize};
use strum_macros::{EnumIter, IntoStaticStr};

#[derive(Debug, Copy, Clone, EnumIter, Hash, PartialEq, Eq, IntoStaticStr, Serialize, Deserialize)]
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
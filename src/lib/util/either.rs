use std::error::Error;
use mlua::{FromLua, FromLuaMulti, IntoLuaMulti, Lua, Value};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Either<A, B> {
    First(A),
    Second(B)
}

impl<A: FromLua, B: FromLua> FromLua for Either<A, B> {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        match A::from_lua(value.clone(), lua) {
            Ok(a) => Ok(Either::First(a)),
            Err(e) => match B::from_lua(value, lua) {
                Ok(b) => Ok(Either::Second(b)),
                Err(e2) => Err(mlua::Error::runtime("Could not parse either value!"))
            }
        }
    }
}
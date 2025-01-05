
use crate::lib::util::{into_lua_value, into_toml_value};
use mlua::prelude::LuaValue;
use mlua::{FromLua, Lua, UserData, UserDataMethods, Value};
use crate::lib::util::cache::Cache;

#[derive(Clone)]
pub struct Storage {
	pub cache: Cache,
}

impl Storage {
	pub(crate) fn new(cache: Cache) -> Storage {
		Storage { cache }
	}
}

impl UserData for Storage {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut(
			"set",
			|_, this, (key, value): (String, LuaValue)| match this
				.cache
				.set_user_value(&key, into_toml_value(&value)?)
			{
				Ok(_) => Ok(()),
				Err(err) => Err(mlua::Error::external(err)),
			},
		);

		methods.add_method_mut("get", |lua, this, key: String| {
			match this.cache.get_user_value(&key) {
				Some(val) => into_lua_value(lua, val),
				None => Ok(LuaValue::Nil),
			}
		})
	}
}

impl FromLua for Storage {
	fn from_lua(
		value: LuaValue,
		_: &Lua,
	) -> mlua::Result<Self> {
		match value {
			Value::UserData(user_data) => {
				if user_data.is::<Self>() {
					Ok(user_data.borrow::<Self>()?.clone())
				} else {
					Err(mlua::Error::UserDataTypeMismatch)
				}
			}

			_ => Err(mlua::Error::UserDataTypeMismatch),
		}
	}
}

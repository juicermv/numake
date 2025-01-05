use mlua::prelude::{LuaFunction, LuaResult, LuaValue};
use mlua::{FromLua, Lua, UserData, UserDataMethods, Value};
use std::collections::HashMap;

#[derive(Clone)]
pub struct TaskManager {
	tasks: HashMap<String, LuaFunction>,
}

impl TaskManager {
	pub fn new() -> Self {
		TaskManager {
			tasks: HashMap::new(),
		}
	}

	pub fn run(
		&self,
		task: &str,
	) -> LuaResult<()> {
		match self.tasks.get(task) {
			Some(t) => t.call::<()>(()),

			None => Err(mlua::Error::RuntimeError(format!(
				"Task {} not found",
				task
			))),
		}
	}

	pub fn get_tasks(&self) -> Vec<String> {
		self.tasks.keys().cloned().collect()
	}
}

impl UserData for TaskManager {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut(
			"create",
			|_, this, (name, task): (String, LuaFunction)| {
				this.tasks.insert(name, task);
				Ok(())
			},
		);
	}
}

impl FromLua for TaskManager {
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

use crate::lib::util::error::NuMakeError::TaskNotFound;
use anyhow::anyhow;
use mlua::prelude::{LuaFunction, LuaResult, LuaValue};
use mlua::{FromLua, Lua, UserData, UserDataMethods, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TaskManager {
	tasks: Arc<Mutex<HashMap<String, LuaFunction>>>,
}

impl TaskManager {
	pub fn new() -> Self {
		TaskManager {
			tasks: Arc::new(Mutex::new(HashMap::new())),
		}
	}

	pub fn run(
		&self,
		task: &str,
	) -> anyhow::Result<()> {
		match (*self.tasks.lock().unwrap()).get(task) {
			Some(t) => match t.call::<()>(()) {
				Ok(_) => Ok(()),
				Err(e) => Err(anyhow!(e)),
			},

			None => Err(anyhow!(TaskNotFound)),
		}
	}

	pub fn get_tasks(&self) -> Vec<String> {
		(*self.tasks.lock().unwrap()).keys().cloned().collect()
	}
}

impl UserData for TaskManager {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut(
			"create",
			|_, this, (name, task): (String, LuaFunction)| {
				(*this.tasks.lock().unwrap()).insert(name, task);
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

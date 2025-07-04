use crate::lib::compilers::generic::Generic;
use crate::lib::compilers::mingw::MinGW;
use crate::lib::compilers::msvc::MSVC;
use crate::lib::compilers::{generic, mingw, msvc};
use crate::lib::data::environment::Environment;
use crate::lib::data::project::Project;
use crate::lib::runtime::filesystem::Filesystem;
use crate::lib::runtime::network::Network;
use crate::lib::runtime::storage::Storage;
use crate::lib::runtime::system::System;
use crate::lib::runtime::task_manager::TaskManager;
use crate::lib::ui::{format, UI};
use crate::lib::util::cache::Cache;
use anyhow::anyhow;
use mlua::prelude::LuaResult;
use mlua::{Compiler, Lua};
use std::fs;

pub mod filesystem;
pub mod network;
pub mod storage;
pub mod system;
pub mod task_manager;

pub struct Runtime {
	// Tools
	task_manager: task_manager::TaskManager,
	network: network::Network,
	storage: storage::Storage,
	filesystem: filesystem::Filesystem,
	system: system::System,

	// Compilers
	msvc: msvc::MSVC,
	mingw: mingw::MinGW,
	generic: generic::Generic,

	ui: UI,
	cache: Cache,
	environment: Environment,

	lua: Lua,
}

impl Runtime {
	pub fn new(
        ui: UI,
		environment: Environment,
	) -> anyhow::Result<Self>{
		let cache: Cache = Cache::new(environment.clone())?;
		let system = System::new(ui.clone());

		Ok(Runtime {
			task_manager: TaskManager::new(),
			network: Network::new(
				environment.clone(),
				ui.clone(),
				cache.clone(),
			),
			storage: Storage::new(cache.clone()),
			filesystem: Filesystem::new(environment.clone()),
			msvc: MSVC::new(environment.clone(), cache.clone(), ui.clone(), system.clone()),
			mingw: MinGW::new(environment.clone(), cache.clone(), ui.clone(), system.clone()),
			generic: Generic::new(
				environment.clone(),
				ui.clone(),
				system.clone(),
			),
			system,
			cache,
			ui,
			environment,
			lua: Lua::new(),
		})
	}

	pub(crate) fn execute_script(
		&mut self,
		filename: &String,
	) -> anyhow::Result<()> {
		let file_size = fs::metadata(filename)?.len();
		let mut should_compile = self.cache.get_value(filename)
			!= Some(toml::Value::from(file_size.to_string()));
		let mut chunk: Vec<u8> = Vec::new();
		false;
		if self.cache.check_file_exists(filename) {
			chunk = self.cache.read_file(filename)?;
		} else {
			should_compile = true;
		}

		self.lua.globals().set("Project", Project::default())?;
		self.lua.globals().set("storage", self.storage.clone())?;
		self.lua
			.globals()
			.set("filesystem", self.filesystem.clone())?;
		self.lua.globals().set("tasks", self.task_manager.clone())?;
		self.lua.globals().set("network", self.network.clone())?;
		self.lua.globals().set("msvc", self.msvc.clone())?;
		self.lua.globals().set("mingw", self.mingw.clone())?;
		self.lua.globals().set("generic", self.generic.clone())?;

		if should_compile {
			chunk = fs::read(filename)?;
			let compiler = Compiler::new()
				.set_optimization_level(2)
				.set_coverage_level(2);
			chunk = compiler.compile(chunk)?;
			self.cache.write_file(filename, &chunk.clone())?;
			self.cache.set_value(
				filename,
				toml::Value::from(chunk.len().to_string()),
			)?;
		}

		self.lua.load(chunk).exec()?;

		self.cache.flush()?;

		Ok(())
	}

	pub fn get_tasks(&mut self) -> Vec<String> {
		self.task_manager.get_tasks()
	}

	pub fn execute_task(
		&mut self,
		task: &str,
	) -> anyhow::Result<()> {
        self.task_manager.run(task)
	}
}

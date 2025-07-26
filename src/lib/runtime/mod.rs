use crate::lib::data::flag_type::FlagType;
use crate::lib::data::project_language::ProjectLanguage;
use crate::lib::data::project_type::ProjectType;
use crate::lib::util::build_cache::BuildCache;
use crate::lib::util::either::Either;
use crate::lib::{
	compilers::{
		generic, generic::Generic, mingw, mingw::MinGW, msvc, msvc::MSVC,
	},
	data::{environment::Environment, project::Project},
	runtime::{
		filesystem::Filesystem, network::Network, storage::Storage,
		system::System, task_manager::TaskManager,
	},
	ui::UI,
	util::cache::Cache,
};
use clap::builder::TypedValueParser;
use mlua::{Compiler, Lua, ObjectLike, Table, Value};
use std::fs;
use std::str::FromStr;

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
	) -> anyhow::Result<Self> {
		let cache: Cache = Cache::new(environment.clone())?;
		let build_cache: BuildCache = BuildCache::new(environment.clone())?;
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
			msvc: MSVC::new(
				environment.clone(),
				build_cache.clone(),
				ui.clone(),
				system.clone(),
			),
			mingw: MinGW::new(
				environment.clone(),
				build_cache.clone(),
				ui.clone(),
				system.clone(),
			),
			generic: Generic::new(
				environment.clone(),
				build_cache.clone(),
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

		if self.cache.check_file_exists(filename) {
			chunk = self.cache.read_file(filename)?;
		} else {
			should_compile = true;
		}

		self.push_globals(&self.lua.globals())?;

		if should_compile {
			chunk = fs::read(filename)?;
			let compiler = Compiler::new()
				.set_optimization_level(2)
				.set_coverage_level(2);
			chunk = compiler.compile(chunk)?;
			self.cache.write_file(filename, chunk.clone())?;
			self.cache.set_value(
				filename,
				toml::Value::from(chunk.len().to_string()),
			)?;
		}

		self.lua.load(chunk).exec()?;

		self.cache.flush()?;

		Ok(())
	}

	fn push_globals(
		&self,
		globals: &Table,
	) -> anyhow::Result<()> {
		globals.set(
			"new_project",
			self.lua.create_function(
				|_, (name, language): (String, ProjectLanguage)| {
					Ok(Project {
						name,
						language,
						..Default::default()
					})
				},
			)?,
		)?;

		

		globals.set("storage", self.storage.clone())?;
		self.lua
			.globals()
			.set("filesystem", self.filesystem.clone())?;
		globals.set("tasks", self.task_manager.clone())?;
		globals.set("network", self.network.clone())?;
		globals.set("msvc", self.msvc.clone())?;
		globals.set("mingw", self.mingw.clone())?;
		globals.set("generic", self.generic.clone())?;

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

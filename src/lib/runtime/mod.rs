use std::fs;
use mlua::{Compiler, Lua};
use mlua::prelude::LuaResult;
use crate::lib::util::cache::Cache;
use crate::lib::compilers::{generic, mingw, msvc};
use crate::lib::compilers::generic::Generic;
use crate::lib::compilers::mingw::MinGW;
use crate::lib::compilers::msvc::MSVC;
use crate::lib::data::environment::Environment;
use crate::lib::data::project::Project;
use crate::lib::runtime::filesystem::Filesystem;
use crate::lib::runtime::network::Network;
use crate::lib::runtime::storage::Storage;
use crate::lib::runtime::system::System;
use crate::lib::runtime::task_manager::TaskManager;
use crate::lib::ui::UI;

pub mod task_manager;
pub mod network;
pub mod storage;
pub mod filesystem;
pub mod system;

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
        environment: Environment,
        quiet: bool,
    ) -> Self {
        let ui: UI = UI::new(quiet);
        let cache: Cache = match Cache::new(environment.clone()) {
            Ok(cache) => cache,
            Err(err) => panic!("{}", err),
        };

        let system = System::new(ui.clone());

        Runtime {
            task_manager: TaskManager::new(),
            network: Network::new(environment.clone(), ui.clone(), cache.clone()),
            storage: Storage::new(cache.clone()),
            filesystem: Filesystem::new(environment.clone()),
            msvc: MSVC::new(environment.clone(), ui.clone(), system.clone()),
            mingw: MinGW::new(environment.clone(), ui.clone(), system.clone()),
            generic: Generic::new(environment.clone(), ui.clone(), system.clone()),
            system,
            cache,
            ui,
            environment,
            lua: Lua::new(),
        }
    }

    pub fn execute_script(&mut self, filename: &String) -> LuaResult<()> {
        let file_size = fs::metadata(filename)?.len();
        let mut should_compile = self.cache.get_value(filename) != Some(toml::Value::from(file_size.to_string()));
        let mut chunk: Vec<u8> = Vec::new();
        false;
        if self.cache.check_file_exists(filename) {
            match self.cache.read_file(filename) {
                Ok(content) => { chunk = content },
                Err(err) => return Err(mlua::Error::external(err))
            }
        } else {
            should_compile = true;
        }

        self.lua.globals().set("Project", Project::default())?;
        self.lua.globals().set("storage", self.storage.clone())?;
        self.lua.globals().set("filesystem", self.filesystem.clone())?;
        self.lua.globals().set("tasks", self.task_manager.clone())?;
        self.lua.globals().set("network", self.network.clone())?;
        self.lua.globals().set("msvc", self.msvc.clone())?;
        self.lua.globals().set("mingw", self.mingw.clone())?;
        self.lua.globals().set("generic", self.generic.clone())?;

        if should_compile {
            chunk = fs::read(filename)?;
            let compiler = Compiler::new().set_optimization_level(2).set_coverage_level(2);
            chunk = compiler.compile(chunk)?;
            match self.cache.write_file(filename, &chunk.clone()) {
                Ok(_) => (),
                Err(err) => return Err(mlua::Error::external(err))
            }

            match self.cache.set_value(filename, toml::Value::from(chunk.len().to_string())) {
                Ok(_) => (),
                Err(err) => return Err(mlua::Error::external(err))
            }
        }

        self.lua.load(chunk).exec()?;

        /*self.task_manager = self.lua.globals().get::<TaskManager>("tasks")?;
        self.storage = self.lua.globals().get::<Storage>("storage")?;

        self.cache.user_values = self.storage.cache.user_values.clone();*/
        self.cache.flush()?;

        Ok(())
    }

    pub fn get_tasks(&mut self) -> Vec<String> {
        self.task_manager.get_tasks()
    }

    pub fn execute_task(&mut self, task: &str) -> LuaResult<()> {
        self.task_manager.run(task)
    }
}
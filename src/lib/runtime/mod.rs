use std::fs;
use mlua::{Chunk, Compiler, Lua};
use mlua::prelude::LuaResult;
use crate::lib::util::cache::Cache;
use crate::lib::compilers::{generic, mingw, msvc};
use crate::lib::compilers::generic::Generic;
use crate::lib::compilers::mingw::MinGW;
use crate::lib::compilers::msvc::MSVC;
use crate::lib::data::environment::Environment;
use crate::lib::data::project::Project;
use crate::lib::runtime::network::Network;
use crate::lib::runtime::storage::Storage;
use crate::lib::runtime::task_manager::TaskManager;
use crate::lib::ui::NumakeUI;
use crate::lib::util::ui::NumakeUI;

pub mod task_manager;
pub mod network;
mod storage;

pub struct Runtime {
    // Tools
    task_manager: task_manager::TaskManager,
    network: network::Network,
    storage: storage::Storage,

    // Compilers
    msvc: msvc::MSVC,
    mingw: mingw::MinGW,
    generic: generic::Generic,

    ui: NumakeUI,
    cache: Cache,
    environment: Environment,

    lua: Lua,
}

impl Runtime {
    pub fn new(
        environment: Environment,
        quiet: bool,
    ) -> Self {
        let ui: NumakeUI = NumakeUI::new(quiet);
        let cache: Cache = match Cache::new(environment.clone()) {
            Ok(cache) => cache,
            Err(err) => panic!("{}", err),
        };

        Runtime {
            task_manager: TaskManager::new(),
            network: Network::new(environment.clone(), ui.clone(), cache.clone()),
            storage: Storage::new(cache.clone()),
            msvc: MSVC::new(environment.clone(), ui.clone()),
            mingw: MinGW::new(environment.clone(), ui.clone()),
            generic: Generic::new(environment.clone(), ui.clone()),
            cache,
            ui,
            environment,
            lua: Lua::new(),
        }
    }

    pub fn execute_script(&mut self, filename: &String) -> LuaResult<()> {
        let mut chunk: Vec<u8> = Vec::new();
        let mut should_compile = false;
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
        self.lua.globals().set("tasks", self.task_manager.clone())?;
        self.lua.globals().set("network", self.network.clone())?;
        self.lua.globals().set("msvc", self.msvc.clone())?;
        self.lua.globals().set("mingw", self.mingw.clone())?;
        self.lua.globals().set("generic", self.generic.clone())?;

        if should_compile {
            chunk = fs::read(filename)?;
            let compiler = Compiler::new().set_optimization_level(2).set_coverage_level(2);
            chunk = compiler.compile(chunk)?;
            match self.cache.write_file(filename, &chunk) {
                Ok(_) => (),
                Err(err) => return Err(mlua::Error::external(err))
            }
        }

        self.lua.load(chunk).exec()?;

        self.task_manager = self.lua.globals().get::<TaskManager>("tasks")?;
        self.storage = self.lua.globals().get::<Storage>("storage")?;

        self.cache.user_values = self.storage.cache.user_values.clone();
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
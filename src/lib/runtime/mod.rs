use anyhow::anyhow;
use mlua::Lua;
use mlua::prelude::LuaResult;
use crate::lib::compilers::{generic, mingw, msvc};
use crate::lib::compilers::generic::Generic;
use crate::lib::compilers::mingw::MinGW;
use crate::lib::compilers::msvc::MSVC;
use crate::lib::data::environment::Environment;
use crate::lib::data::project::Project;
use crate::lib::runtime::network::Network;
use crate::lib::runtime::task_manager::TaskManager;
use crate::lib::ui::NumakeUI;

pub mod task_manager;
pub mod network;

pub struct Runtime {
    // Tools
    task_manager:  task_manager::TaskManager,
    network:  network::Network,


    // Compilers
    msvc:  msvc::MSVC,
    mingw:  mingw::MinGW,
    generic:  generic::Generic,

    ui:  NumakeUI,

    lua: Lua,
}

impl Runtime {
    pub  fn new(
        environment:  Environment,
    ) -> Self {
        let ui:  NumakeUI =  NumakeUI::new(false);

        Runtime {
            task_manager:  TaskManager::new(),
            network:  Network::new(environment.clone(), ui.clone()),
            msvc:  MSVC::new(environment.clone(), ui.clone()),
            mingw:  MinGW::new(environment.clone(), ui.clone()),
            generic:  Generic::new(environment.clone(), ui.clone()),
            ui,
            lua: Lua::new(),
        }
    }

    pub fn execute_script(&mut self, code: &str) -> LuaResult<()> {
        self.lua.globals().set("Project", Project::default())?;
        self.lua.globals().set("tasks", self.task_manager.clone())?;
        self.lua.globals().set("network", self.network.clone())?;
        self.lua.globals().set("msvc", self.msvc.clone())?;
        self.lua.globals().set("mingw", self.mingw.clone())?;
        self.lua.globals().set("generic", self.generic.clone())?;

        self.lua.load(code).exec()?;

        self.task_manager = self.lua.globals().get::<TaskManager>("tasks")?;

        Ok(())
    }

    pub fn get_tasks(&mut self) -> Vec<String> {
        self.task_manager.get_tasks()
    }

    pub fn execute_task(&mut self, task: &str) -> LuaResult<()> {
        self.task_manager.run(task)
    }
}
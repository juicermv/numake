use anyhow::anyhow;
use mlua::{
	prelude::{LuaError, LuaValue},
	Compiler, FromLua, Lua, UserData, UserDataFields, UserDataMethods, Value,
};
use serde::Serialize;
use std::ptr::null;
use std::{collections::HashMap, fs, io::Cursor, path::PathBuf};
use zip::ZipArchive;

use crate::lib::cache::Cache;
use crate::lib::cli_args::{InspectArgs, ListArgs, NuMakeArgs};
use crate::lib::error::NuMakeError::{
	PathOutsideWorkingDirectory, TargetNotFound,
};
use crate::lib::target::custom::CustomTarget;
use crate::lib::target::generic::GenericTarget;
use crate::lib::target::mingw::MinGWTarget;
use crate::lib::target::msvc::MSVCTarget;
use crate::lib::target::{Target, TargetTrait};
use crate::lib::ui::NumakeUI;
use crate::lib::util::{
	args_to_map, into_lua_value, into_toml_value, to_lua_result,
};

#[derive(Clone, Serialize, Default)]
pub struct LuaWorkspace {
	pub(crate) workspace: PathBuf,
	pub(crate) working_directory: PathBuf, // Should already exist

	pub(crate) output: Option<String>,

	pub(crate) toolset_compiler: Option<String>,
	pub(crate) toolset_linker: Option<String>,

	#[serde(skip_serializing)]
	pub cache: Cache,

	targets: HashMap<String, Target>,
	file: PathBuf,
	arguments: Vec<String>,

	#[serde(skip_serializing)]
	ui: NumakeUI,

	#[serde(skip_serializing)]
	target: String,
}

impl UserData for LuaWorkspace {
	fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
		fields.add_field_method_get("arguments", |lua, this| {
			lua.create_table_from(args_to_map(this.arguments.clone()))
		});

		fields.add_field_method_get("env", |_, _| {
			Ok(std::env::vars().collect::<HashMap<String, String>>())
		});

		fields
			.add_field_method_get("platform", |_, _| Ok(std::env::consts::OS));

		fields.add_field_method_get("arch", |_, _| Ok(std::env::consts::ARCH));
	}

	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method("create_target", |_, this, name: String| {
			to_lua_result(GenericTarget::new(
				name,
				this.toolset_compiler.clone(),
				this.toolset_linker.clone(),
				this.output.clone(),
				this.working_directory.clone(),
				this.ui.clone(),
			))
		});

		methods.add_method(
			"create_custom_target",
			|_, _, (name, description): (String, String)| {
				Ok(CustomTarget::new(name, description))
			},
		);

		methods.add_method("create_msvc_target", |_, this, name: String| {
			to_lua_result(MSVCTarget::new(
				name,
				this.output.clone(),
				this.working_directory.clone(),
				this.ui.clone(),
			))
		});

		methods.add_method("create_mingw_target", |_, this, name: String| {
			to_lua_result(MinGWTarget::new(
				name,
				this.output.clone(),
				this.working_directory.clone(),
				this.ui.clone(),
			))
		});

		methods.add_method_mut("register_target", |_, this, target: Target| {
			Ok(this.targets.insert(target.get_name(), target))
		});

		methods.add_method_mut("download_zip", |_, this, url: String| {
			to_lua_result(this.workspace_download_zip(url))
		});

		methods.add_method_mut("load_url", |lua, this, url: String| {
			let chunk = to_lua_result(this.load_url(&url))?;
			lua.load(chunk).set_name(url).eval::<LuaValue>()
		});

		methods.add_method_mut("load", |lua, this, path: String| {
			let chunk = to_lua_result(this.load(&path))?;
			lua.load(chunk).set_name(path).eval::<LuaValue>()
		});

		methods.add_method_mut(
			"set",
			|_, this, (key, value): (String, LuaValue)| {
				this.cache.user_values.insert(key, into_toml_value(&value)?);
				Ok(())
			},
		);

		methods.add_method("get", |lua, this, key: String| {
			if this.cache.user_values.contains_key(&key) {
				Ok(Some(into_lua_value(
					lua.clone(),
					this.cache.user_values.get(&key).unwrap(),
				)?))
			} else {
				Ok(None)
			}
		});

		methods.add_method(
			"walk_dir",
			|_,
			 this,
			 (path, recursive, filter): (String, bool, Option<Vec<String>>)| {
				let paths = to_lua_result(this.walk_dir(
					dunce::canonicalize(this.working_directory.join(path))?,
					recursive,
					&filter,
				))?
				.clone();

				let paths_str = paths
					.iter()
					.map(|path| path.to_str().unwrap_or_default().to_string())
					.collect::<Vec<String>>();

				Ok(paths_str.clone())
			},
		);
	}
}

impl LuaWorkspace {
	pub fn new(args: &NuMakeArgs) -> anyhow::Result<Self> {
		let workdir = dunce::canonicalize(&args.workdir)?;
		let workspace = workdir.join(".numake");
		let file = dunce::canonicalize(workdir.join(&args.file))?;

		if !file.starts_with(&workdir) {
			Err(anyhow!(PathOutsideWorkingDirectory))?;
		}

		Ok(LuaWorkspace {
			targets: HashMap::new(),
			working_directory: workdir,
			file,
			workspace: workspace.clone(),
			target: args.target.clone(),
			toolset_compiler: args.toolset_compiler.clone(),
			toolset_linker: args.toolset_linker.clone(),
			output: args.output.clone(),

			arguments: args.arguments.clone().unwrap_or_default(),
			ui: NumakeUI::new(args.quiet),
			cache: Cache::new(workspace)?,
		})
	}

	pub fn new_inspect(args: &InspectArgs) -> anyhow::Result<Self> {
		let workdir = dunce::canonicalize(&args.workdir)?;
		let workspace = workdir.join(".numake");
		let file = dunce::canonicalize(workdir.join(&args.file))?;

		if !file.starts_with(&workdir) {
			Err(anyhow!(PathOutsideWorkingDirectory))?;
		}

		Ok(LuaWorkspace {
			targets: HashMap::new(),
			working_directory: workdir,
			file,
			workspace: workspace.clone(),
			target: "*".to_string(),
			toolset_compiler: args.toolset_compiler.clone(),
			toolset_linker: args.toolset_linker.clone(),
			output: args.output.clone(),

			arguments: args.arguments.clone().unwrap_or_default(),
			ui: NumakeUI::new(args.quiet),
			cache: Cache::new(workspace)?,
		})
	}

	pub fn new_dummy(args: &ListArgs) -> anyhow::Result<Self> {
		let workdir = dunce::canonicalize(&args.workdir)?;
		let workspace = workdir.join(".numake");
		let file = dunce::canonicalize(workdir.join(&args.file))?;

		if !file.starts_with(&workdir) {
			Err(anyhow!(PathOutsideWorkingDirectory))?;
		}

		Ok(LuaWorkspace {
			targets: HashMap::new(),
			working_directory: workdir,
			file,
			workspace: workspace.clone(),
			target: "".to_string(),
			toolset_compiler: None,
			toolset_linker: None,
			output: None,

			arguments: args.arguments.clone().unwrap_or_default(),
			ui: NumakeUI::new(args.quiet),
			cache: Cache::new(workspace)?,
		})
	}

	pub fn process(
		&mut self,
		lua_state: &Lua,
	) -> anyhow::Result<()> {
		let spinner = self.ui.spinner("Processing script...".to_string());
		std::env::set_current_dir(&self.working_directory)?;

		lua_state.set_compiler(
			Compiler::new()
				.set_debug_level(2)
				.set_coverage_level(2)
				.set_optimization_level(0),
		);

		if !self.workspace.exists() {
			fs::create_dir_all(&self.workspace)?;
		}

		if !self.file.starts_with(&self.working_directory) {
			// Throw error if file is outside working directory
			Err(anyhow!(PathOutsideWorkingDirectory))?
		}

		match lua_state.globals().set("workspace", self.clone()) {
			Err(e) => return Err(anyhow!(e.to_string())),
			Ok(_) => (),
		};

		// Custom print function
		match lua_state.create_function_mut(|lua, out: LuaValue| {
			let workspace = lua.globals().get::<LuaWorkspace>("workspace")?;
			workspace
				.ui
				.progress_manager
				.println(workspace.ui.format_info(out.to_string()?))?;
			Ok(())
		}) {
			Err(e) => return Err(anyhow!(e.to_string())),
			Ok(func) => match lua_state.globals().set("print", func) {
				Err(e) => return Err(anyhow!(e.to_string())),
				Ok(_) => (),
			},
		}

		// Caching
		let file_size = self.file.metadata()?.len().to_string();
		let file_size_toml = toml::Value::from(file_size.clone());
		let file_name = &self.file.to_str().unwrap().to_string();
		let file_cache_exists: bool = self.cache.check_key_exists(file_name)
			&& self.cache.check_file_exists(file_name)
			&& self.cache.get_value(file_name).is_some();
		let cached_file_size = if file_cache_exists {
			self.cache.get_value(file_name).unwrap().clone()
		} else {
			toml::Value::String("-1".to_string())
		};

		if file_cache_exists && cached_file_size == file_size_toml {
			match lua_state
				.load(self.cache.read_file(file_name)?)
				.set_name(self.file.file_name().unwrap().to_str().unwrap())
				.exec()
			{
				Err(e) => return Err(anyhow!(e.to_string())),
				Ok(_) => (),
			}
		} else if cached_file_size != file_size_toml || !file_cache_exists {
			let file_content = fs::read(&self.file)?;
			match lua_state
				.load(&file_content)
				.set_name(self.file.file_name().unwrap().to_str().unwrap())
				.exec()
			{
				Err(e) => return Err(anyhow!(e.to_string())),
				Ok(_) => (),
			}

			match Compiler::new()
				.set_debug_level(0)
				.set_optimization_level(2)
				.set_coverage_level(2)
				.compile(&file_content)
			{
				Err(e) => return Err(anyhow!(e.to_string())),
				Ok(bytes) => match self.cache.write_file(file_name, bytes) {
					Err(e) => return Err(anyhow!(e.to_string())),
					Ok(_) => (),
				},
			}

			self.cache.set_value(file_name, file_size_toml.clone())?;
		}

		// Read back workspace values from Lua
		let mut lua_workspace: LuaWorkspace = LuaWorkspace::default();
		match lua_state.globals().get("workspace") {
			Err(e) => return Err(anyhow!(e.to_string())),
			Ok(val) => lua_workspace = val,
		}

		self.targets = lua_workspace.targets.clone();
		self.ui.progress_manager = lua_workspace.ui.progress_manager;
		self.cache.user_values = lua_workspace.cache.user_values;

		for name in self.targets.clone().keys() {
			let mut target = self.targets[name].clone();
			target.set_vscode_props(&mut self.clone())?;
			self.targets.insert(name.clone(), target);
		}
		// Write cache to disk
		self.cache.flush()?;

		spinner.finish_and_clear();
		self.ui
			.progress_manager
			.println(self.ui.format_info(format!(
				"Processing script done in {}ms.",
				spinner.elapsed().as_millis()
			)))?;

		Ok(())
	}

	pub fn list_targets(&self) -> anyhow::Result<String> {
		Ok(self
			.targets
			.iter()
			.map(|(name, target)| match target {
				Target::Generic(_) => {
					format!("\n\t{} [GENERIC]", name)
				}
				Target::MSVC(_) => {
					format!("\n\t{} [MSVC]", name)
				}
				Target::MinGW(_) => {
					format!("\n\t{} [MINGW]", name)
				}
				Target::Custom(target) => {
					format!("\n\t{} [CUSTOM]\n{}", name, target.description)
				}
			})
			.collect())
	}

	pub fn build(&mut self) -> anyhow::Result<()> {
		let mut result = Ok(());
		if self.target == "all" || self.target == "." {
			for (target, _) in self.targets.clone() {
				self.build_target(&target)?;
			}
		} else {
			result = self.build_target(&self.target.clone());
		}
		result
	}

	fn build_target(
		&mut self,
		_target: &String,
	) -> anyhow::Result<()> {
		if !self.targets.contains_key(_target) {
			Err(anyhow!(TargetNotFound))
		} else {
			let spinner =
				self.ui.spinner(format!("Building target {}...", _target));
			let result =
				self.targets.get(_target).unwrap().build(&mut self.clone());
			if result.is_ok() {
				spinner.finish_and_clear();
				self.ui.progress_manager.println(self.ui.format_info(
					format!(
						"Building target {} done in {}ms.",
						_target,
						spinner.elapsed().as_millis()
					),
				))?;
				Ok(())
			} else {
				let err = result.err().unwrap();
				spinner.finish_and_clear();
				self.ui.progress_manager.println(self.ui.format_err(
					format!("Building target {} FAILED!", _target,),
				))?;
				Err(err)
			}
		}
	}

	fn load(
		&mut self,
		path: &String,
	) -> anyhow::Result<Vec<u8>> {
		let file = self.working_directory.join(path);
		if file.starts_with(&self.working_directory) && file.exists() {
			let file_size = file.metadata()?.len().to_string();
			let file_size_toml = toml::Value::from(file_size.clone());
			let file_name = &file.to_str().unwrap().to_string();
			let file_cache_exists: bool =
				self.cache.check_key_exists(file_name)
					&& self.cache.check_file_exists(file_name)
					&& self.cache.get_value(file_name).is_some();
			let cached_file_size = if file_cache_exists {
				self.cache.get_value(file_name).unwrap().clone()
			} else {
				toml::Value::String("-1".to_string())
			};

			if file_cache_exists && cached_file_size == file_size_toml {
				Ok(self.cache.read_file(file_name)?)
			} else {
				let file_content = fs::read(&file)?;
				match Compiler::new()
					.set_debug_level(0)
					.set_optimization_level(2)
					.set_coverage_level(2)
					.compile(&file_content)
				{
					Err(e) => Err(anyhow!(e.to_string())),
					Ok(bytes) => {
						self.cache.write_file(file_name, &bytes)?;
						self.cache
							.set_value(file_name, file_size_toml.clone())?;
						Ok(bytes)
					}
				}
			}
		} else {
			Err(anyhow!(PathOutsideWorkingDirectory))
		}
	}

	fn load_url(
		&mut self,
		url: &String,
	) -> anyhow::Result<Vec<u8>> {
		let file_cache_exists: bool = self.cache.check_key_exists(url)
			&& self.cache.check_file_exists(url)
			&& self.cache.get_value(url).is_some();

		let response = reqwest::blocking::get(url)?;
		if !response.status().is_success() {
			if file_cache_exists {
				Ok(self.cache.read_file(url)?)
			} else {
				Err(anyhow!(response.status()))
			}
		} else {
			let file_size = response.content_length().unwrap_or(0).to_string();
			let file_size_toml = toml::Value::from(file_size.clone());
			let cached_file_size = if file_cache_exists {
				self.cache.get_value(url).unwrap().clone()
			} else {
				toml::Value::String("-1".to_string())
			};

			if file_cache_exists && cached_file_size == file_size_toml {
				Ok(self.cache.read_file(url)?)
			} else {
				let file_content = response.bytes()?;

				self.cache.set_value(url, file_size_toml)?;
				match Compiler::new()
					.set_debug_level(0)
					.set_optimization_level(2)
					.set_coverage_level(2)
					.compile(&file_content)
				{
					Err(e) => Err(anyhow!(e.to_string())),
					Ok(bytes) => {
						self.cache.write_file(url, bytes)?;

						return Ok(file_content.to_vec());
					}
				}
			}
		}
	}

	fn workspace_download_zip(
		&mut self,
		url: String,
	) -> anyhow::Result<String> {
		if self.cache.check_dir_exists(&url) {
			self.ui
				.progress_manager
				.println(self.ui.format_info(format!(
				"Cache entry for [{}] found. \nNot downloading. This is okay!",
				url
			)))?;
			Ok(self.cache.get_dir(&url)?.to_str().unwrap().to_string())
		} else {
			let response = reqwest::blocking::get(&url)?;
			if response.status().is_success() {
				let spinner = self
					.ui
					.spinner("Downloading & extracting archive...".to_string());
				self.ui.progress_manager.println(self.ui.format_ok(
					format!(
						"Server responded with {}! [{}]",
						response.status(),
						&url
					),
				))?;
				let path = self.cache.get_dir(&url)?;

				ZipArchive::new(Cursor::new(response.bytes()?))?
					.extract(&path)?;
				spinner.finish_and_clear();
				self.ui.progress_manager.println(
					self.ui.format_ok(format!("Done extracting! [{}]", url)),
				)?;

				Ok(path.to_str().unwrap().to_string())
			} else {
				Err(anyhow!(response.status()))?
			}
		}
	}

	pub fn walk_dir(
		&self,
		path_buf: PathBuf,
		recursive: bool,
		filter: &Option<Vec<String>>,
	) -> anyhow::Result<Vec<PathBuf>> {
		let mut path_vec: Vec<PathBuf> = Vec::new();

		if !path_buf.starts_with(&self.working_directory) {
			return Err(anyhow!(PathOutsideWorkingDirectory));
		}

		for entry in fs::read_dir(path_buf)? {
			let path = dunce::canonicalize(entry?.path())?;
			if path.is_dir() && recursive {
				path_vec.append(
					&mut self.walk_dir(path.clone(), true, filter)?.clone(),
				)
			}
			if path.is_file() {
				if !filter.is_none() {
					if filter.clone().unwrap().contains(
						&path
							.extension()
							.unwrap_or("".as_ref())
							.to_str()
							.unwrap()
							.to_string(),
					) {
						path_vec.push(path.clone());
					}
				} else {
					path_vec.push(path.clone());
				}
			}
		}

		Ok(path_vec)
	}
}

impl FromLua for LuaWorkspace {
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

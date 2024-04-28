use std::{
	collections::HashMap,
	fs,
	path::{
		Path,
		PathBuf,
	},
	str::FromStr,
	time::SystemTime,
};
use std::io::Cursor;

use anyhow::anyhow;
use mlua::{
	Compiler,
	FromLua,
	Lua,
	UserData,
	UserDataFields,
	UserDataMethods,
	Value,
};
use serde::Serialize;
use uuid::Uuid;
use zip::ZipArchive;

use crate::{
	config::{
		InspectArgs,
		ListArgs,
		NuMakeArgs,
	},
	error::{
		NUMAKE_ERROR,
		to_lua_result,
	},
	target::Target,
	util::log,
};

#[derive(Clone, Serialize)]
pub struct LuaFile
{
	pub(crate) workspace: PathBuf,
	pub(crate) workdir: PathBuf, // Should already exist

	pub(crate) output: Option<String>,

	pub(crate) toolset_compiler: Option<String>,
	pub(crate) toolset_linker: Option<String>,

	targets: HashMap<String, Target>,
	file: PathBuf,
	arguments: Vec<String>,

	#[serde(skip_serializing)]
	quiet: bool,
	#[serde(skip_serializing)]
	target: String,
}

impl UserData for LuaFile
{
	fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F)
	{
		fields.add_field_method_get("arguments", |_, this| {
			Ok(this.arguments.clone())
		});
	}

	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M)
	{
		methods.add_method("create_target", |_, this, name: String| {
			to_lua_result(Target::new(
				name.clone(),
				this.toolset_compiler.clone(),
				this.toolset_linker.clone(),
				this.output.clone(),
				this.workdir.clone(),
				false,
				this.quiet,
			))
		});

		methods.add_method("create_msvc_target", |_, this, name: String| {
			to_lua_result(Target::new(
				name.clone(),
				None,
				None,
				this.output.clone(),
				this.workdir.clone(),
				true,
				this.quiet,
			))
		});

		methods.add_method_mut("register_target", |_, this, target: Target| {
			Ok(this.targets.insert(target.name.clone(), target))
		});

		methods.add_method("download_zip", |_, this, url: String| {
			to_lua_result(this.workspace_download_zip(url))
		});

		methods.add_method("require_url", |lua, this, url: String| {
			to_lua_result(this.require_url(lua, url))
		});
	}
}

impl<'lua> FromLua<'lua> for LuaFile
{
	fn from_lua(
		value: Value<'lua>,
		_: &'lua Lua,
	) -> mlua::Result<Self>
	{
		match value {
			Value::UserData(user_data) => {
				Ok(user_data.borrow::<Self>()?.clone())
			}
			_ => unreachable!(),
		}
	}
}

impl LuaFile
{
	pub fn new(args: &NuMakeArgs) -> anyhow::Result<Self>
	{
		Ok(LuaFile {
			targets: HashMap::new(),
			workdir: dunce::canonicalize(&args.workdir)?,
			file: dunce::canonicalize(
				dunce::canonicalize(&args.workdir)?.join(&args.file),
			)?,
			workspace: dunce::canonicalize(&args.workdir)?.join(".numake"),
			target: args.target.clone(),
			toolset_compiler: args.toolset_compiler.clone(),
			toolset_linker: args.toolset_linker.clone(),
			output: args.output.clone(),

			arguments: args.arguments.clone().unwrap_or_default(),
			quiet: args.quiet,
		})
	}

	pub fn new_inspect(args: &InspectArgs) -> anyhow::Result<Self>
	{
		Ok(LuaFile {
			targets: HashMap::new(),
			workdir: dunce::canonicalize(&args.workdir)?,
			file: dunce::canonicalize(
				dunce::canonicalize(&args.workdir)?.join(&args.file),
			)?,
			workspace: dunce::canonicalize(&args.workdir)?.join(".numake"),
			target: "*".to_string(),
			toolset_compiler: args.toolset_compiler.clone(),
			toolset_linker: args.toolset_linker.clone(),
			output: args.output.clone(),

			arguments: args.arguments.clone().unwrap_or_default(),
			quiet: args.quiet,
		})
	}

	pub fn new_dummy(args: &ListArgs) -> anyhow::Result<Self>
	{
		Ok(LuaFile {
			targets: HashMap::new(),
			workdir: dunce::canonicalize(&args.workdir)?,
			file: dunce::canonicalize(
				dunce::canonicalize(&args.workdir)?.join(&args.file),
			)?,
			workspace: dunce::canonicalize(&args.workdir)?.join(".numake"),
			target: "".to_string(),
			toolset_compiler: None,
			toolset_linker: None,
			output: None,

			arguments: vec![],
			quiet: args.quiet,
		})
	}

	pub fn process(
		&mut self,
		lua_state: &Lua,
	) -> anyhow::Result<()>
	{
		let now = SystemTime::now();
		lua_state.set_compiler(
			Compiler::new()
				.set_debug_level(2)
				.set_coverage_level(2)
				.set_optimization_level(0),
		);

		if !self.workspace.exists() {
			fs::create_dir_all(&self.workspace)?;
		}

		if !self.file.starts_with(&self.workdir) {
			// Throw error if file is outside working directory
			Err(anyhow!(&NUMAKE_ERROR.PATH_OUTSIDE_WORKING_DIR))?
		}

		lua_state.globals().set("workspace", self.clone())?;

		let file_uuid = Uuid::new_v8(
			*self
				.file
				.to_str()
				.unwrap()
				.as_bytes()
				.last_chunk::<16>()
				.unwrap(),
		)
		.to_string();

		let cache_dir = self.workspace.join("cache");
		if !cache_dir.exists() {
			fs::create_dir_all(&cache_dir)?;
		}

		let cache_toml = cache_dir.join("cache.toml");
		let file_size = self.file.metadata()?.len().to_string();
		let file_size_toml = toml::Value::from(file_size.clone());
		if !cache_toml.exists() {
			fs::write(&cache_toml, "")?;
		}

		let mut table =
			toml::Table::from_str(&fs::read_to_string(&cache_toml)?)?;
		if !table.contains_key(&file_uuid) {
			table.insert(file_uuid.clone(), file_size_toml.clone());
		}

		let file_cache =
			cache_dir.join(format!("{}.{}", &file_uuid, "nucache"));

		if table[&file_uuid] == file_size_toml && file_cache.exists() {
			log("Loading and executing script from cache...", self.quiet);
			lua_state
				.load(fs::read(&file_cache)?)
				.set_name(self.file.file_name().unwrap().to_str().unwrap())
				.exec()?;
			log("Success!", self.quiet);
		} else if table[&file_uuid] != file_size_toml || !file_cache.exists() {
			let file_content = fs::read(&self.file)?;
			log("Loading and executing script...", self.quiet);
			lua_state
				.load(&file_content)
				.set_name(self.file.file_name().unwrap().to_str().unwrap())
				.exec()?;
			log("Success! Saving script to cache...", self.quiet);
			fs::write(
				&file_cache,
				Compiler::new()
					.set_debug_level(0)
					.set_optimization_level(2)
					.set_coverage_level(2)
					.compile(&file_content),
			)?;

			table[&file_uuid] = file_size_toml.clone();
			fs::write(&cache_toml, table.to_string())?;
			log("Done.", self.quiet);
		}

		let lua_workspace: Self = lua_state.globals().get("workspace")?;
		self.targets = lua_workspace.targets.clone();

		log(
			&format!(
				"Processing script done in {}ms!",
				now.elapsed()?.as_millis()
			),
			self.quiet,
		);

		Ok(())
	}

	pub fn list_targets(&self) -> anyhow::Result<String>
	{
		Ok(self
			.targets
			.iter()
			.map(|(name, target)| {
				if !target.is_msvc() {
					format!("{}: generic", name)
				} else {
					format!("{}: msvc", name)
				}
			})
			.collect())
	}

	pub fn build(&mut self) -> anyhow::Result<()>
	{
		if self.target == "all" || self.target == "*" {
			for (target, _) in self.targets.clone() {
				self.build_target(&target)?;
			}
			Ok(())
		} else {
			self.build_target(&self.target.clone())
		}
	}

	fn build_target(
		&self,
		_target: &String,
	) -> anyhow::Result<()>
	{
		if !self.targets.contains_key(_target) {
			Err(anyhow!(&NUMAKE_ERROR.TARGET_NOT_FOUND))
		} else {
			log(&format!("Selecting target {}...", _target), self.quiet);
			log(&format!("Building target {}...", _target), self.quiet);
			let now = SystemTime::now();
			self.targets.get(_target).unwrap().build(self)?;
			log(
				&format!(
					"Building target {} done in {}ms!",
					_target,
					now.elapsed()?.as_millis()
				),
				self.quiet,
			);
			Ok(())
		}
	}

	fn require_url(
		&self,
		lua_state: &Lua,
		url: String,
	) -> anyhow::Result<()>
	{
		let file_uuid =
			Uuid::new_v8(*url.as_bytes().last_chunk::<16>().unwrap())
				.to_string();
		let cache_dir = self.workspace.join("cache");
		let cache_toml = cache_dir.join("cache.toml");
		if !cache_toml.exists() {
			fs::write(&cache_toml, format!("{}=\"-1\"", &file_uuid))?;
		}

		let mut table =
			toml::Table::from_str(&fs::read_to_string(&cache_toml)?)?;
		let file_cache =
			cache_dir.join(format!("{}.{}", &file_uuid, "nucache"));

		let response = reqwest::blocking::get(&url)?;
		if !response.status().is_success() {
			if table.contains_key(&file_uuid) && file_cache.exists() {
				Ok(lua_state
					.load(fs::read(&file_cache)?)
					.set_name(&url)
					.exec()?)
			} else {
				Err(anyhow!(response.status()))?
			}
		} else {
			let file_size = response.content_length().unwrap_or(0).to_string();
			let file_size_toml = toml::Value::from(file_size.clone());
			if !table.contains_key(&file_uuid) {
				table.insert(file_uuid.clone(), toml::Value::from("-1"));
			}

			if table[&file_uuid] == file_size_toml && file_cache.exists() {
				Ok(lua_state
					.load(fs::read(&file_cache)?)
					.set_name(&url)
					.exec()?)
			} else if table[&file_uuid] != file_size_toml
				|| !file_cache.exists()
			{
				let file_content = response.text()?;
				lua_state.load(&file_content).set_name(&url).eval()?;
				fs::write(
					&file_cache,
					Compiler::new()
						.set_debug_level(0)
						.set_optimization_level(2)
						.set_coverage_level(2)
						.compile(&file_content),
				)?;
				table[&file_uuid] = file_size_toml;
				fs::write(&cache_toml, table.to_string())?;
				Ok(())
			} else {
				Err(anyhow!("URL REQUIRE ERROR"))?
			}
		}
	}

	fn workspace_download_zip(
		&self,
		url: String,
	) -> anyhow::Result<String>
	{
		log("Starting zip download...", self.quiet);
		let path_str: String = format!(
			// Where the archive will be extracted.
			"{}/remote/{}",
			self.workspace.to_str().unwrap_or("ERROR"),
			Uuid::new_v8(*url.as_bytes().last_chunk::<16>().unwrap())
		);

		let path = Path::new(&path_str);

		if path.exists() && path.is_dir() {
			log(&format!("Found non-empty extract path on system! ({}) Not downloading. (This is okay!)", &path_str), self.quiet);
			Ok(path.to_str().unwrap().to_string())
		} else {
			let response = reqwest::blocking::get(&url)?;
			if response.status().is_success() {
				log(
					&format!(
						"Server responded with {}!",
						response.status().to_string()
					),
					self.quiet,
				);
				fs::create_dir_all(path)?;
				log("Downloading & extracting archive...", self.quiet);
				ZipArchive::new(Cursor::new(response.bytes()?))?.extract(path)?;
				log("Done!", self.quiet);
				Ok(path.to_str().unwrap().to_string())
			} else {
				Err(anyhow!(response.status()))?
			}
		}
	}
}

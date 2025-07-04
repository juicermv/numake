use std::{
	fs,
	path::PathBuf,
	process::Command,
};
use std::collections::{HashMap, HashSet};
use crate::lib::data::environment::Environment;
use crate::lib::data::project::Project;
use crate::lib::data::project_language::ProjectLanguage;
use crate::lib::data::project_type::ProjectType;
use crate::lib::data::source_file_type::SourceFileType;
use crate::lib::runtime::system::System;
use crate::lib::ui::UI;
use mlua::{prelude::LuaValue, FromLua, Lua, UserData, UserDataMethods, Value};
use pathdiff::diff_paths;
use crate::lib::util::cache::Cache;

#[derive(Clone)]
pub struct MinGW {
	environment: Environment,
	cache: Cache,
	ui: UI,
	system: System,
}

impl MinGW {
	pub fn new(
		environment: Environment,
		cache: Cache,
		ui: UI,
		system: System,
	) -> Self {
		MinGW {
			environment,
			cache,
			ui,
			system,
		}
	}

	fn compile_step(
		&mut self,
		project: &Project,
		obj_dir: &PathBuf,
		mingw: &String,
		o_files: &mut Vec<String>,
	) -> anyhow::Result<()> {
		let source_files = project.source_files.get(&SourceFileType::Code);

		let binding = self
			.cache
			.get_value("mingw_cache")
			.unwrap_or(toml::Value::Array(Vec::new()));

		let binding2: Vec<toml::Value> = Vec::new();

		/*
		 * We cache the hashes of files that have been previously compiled
		 * to figure out whether we should compile them again.
		 */
		let mut mingw_cache: HashSet<String> = binding
			.as_array()
			.unwrap_or(&binding2)
			.iter()
			.map(|value| value.as_str().unwrap().to_string())
			.collect::<HashSet<String>>();

		/*
         * Hash the contents of every source file once
         * so we don't have to do it multiple times.
         */
		let hashes: HashMap<&PathBuf, String> = source_files
			.iter()
			.filter_map(|file| match sha256::try_digest(file) {
				Ok(digest) => Some((file, digest)),
				Err(_) => None,
			})
			.collect();

		let dirty_files: Vec<&PathBuf> = source_files
			.iter()
			.filter(|file| match hashes.get(file) {
				Some(hash) => {
					!mingw_cache.contains(hash)
				}

				None => true,
			})
			.collect();

		let clean_hashes: HashSet<String> = source_files
			.iter()
			.filter_map(|file| match hashes.get(file) {
				Some(hash) => {
					if mingw_cache.contains(hash) {
						Some(hash.clone())
					} else {
						None
					}
				}

				None => None,
			})
			.collect();

		let progress = self.ui.create_bar(dirty_files.len() as u64, "Compiling... ");

		mingw_cache = clean_hashes;

		// COMPILATION STEP
		for file in source_files.clone() {
			let o_file = obj_dir.join(
				diff_paths(&file, &(self.environment).project_directory)
					.unwrap()
					.to_str()
					.unwrap()
					.to_string() + ".o",
			);

			o_files.push(o_file.to_str().unwrap().to_string());

			if !dirty_files.contains(&&file) {
				continue;
			}

			progress.inc(1);
			progress.set_message(
				"Compiling... ".to_string() + file.to_str().unwrap(),
			);
			let mut compiler = Command::new(
				mingw.clone()
					+ match project.language {
						ProjectLanguage::C => "gcc",
						ProjectLanguage::CPP => "g++",
					},
			);



			if !o_file.parent().unwrap().exists() {
				fs::create_dir_all(o_file.parent().unwrap())?;
			}

			let mut compiler_args = vec![
				"-c".to_string(),
				format!("-o{}", o_file.to_str().unwrap()),
			];



			for incl in project.include_paths.clone() {
				compiler_args.push(format!("-I{incl}"))
			}

			for define in project.defines.clone() {
				compiler_args.push(format!("-D{define}"))
			}

			for flag in project.compiler_flags.clone() {
				compiler_args.push(flag)
			}

			compiler_args.push(file.to_str().unwrap_or("ERROR").to_string());

			match self.system.execute(
				compiler
					.args(&compiler_args)
					.current_dir(&(self.environment).project_directory),
			) {
				Ok(status) => {
					mingw_cache.insert(hashes[&file].clone());
					Ok(status)
				}

				Err(err) => Err(err)
			}?;
		}

		self.ui.remove_bar(progress);

		let mingw_cache_toml: toml::Value = toml::Value::Array(
			mingw_cache
				.iter()
				.map(|value| toml::Value::String(value.clone()))
				.collect::<Vec<toml::Value>>(),
		);

		self.cache.set_value("mingw_cache", mingw_cache_toml)?;
		self.cache.flush()?;

		Ok(())
	}

	fn resource_step(
		&mut self,
		project: &Project,
		mingw: &String,
		res_dir: &PathBuf,
		o_files: &mut Vec<String>,
	) -> anyhow::Result<()> {
		let resource_files =
			project.source_files.get(&SourceFileType::Resource);
		let progress = self.ui.create_bar(resource_files.len() as u64, "Compiling Resources... ");
		// RESOURCE FILE HANDLING
		for resource_file in resource_files {
			progress.inc(1);
			progress.set_message(
				"Compiling Resources... ".to_string()
					+ resource_file.to_str().unwrap(),
			);
			let mut resource_compiler = Command::new(mingw.clone() + "windres");

			let coff_file = res_dir.join(
				diff_paths(
					&resource_file,
					&(self.environment).project_directory,
				)
				.unwrap()
				.to_str()
				.unwrap()
				.to_string() + ".o",
			);

			if !coff_file.parent().unwrap().exists() {
				fs::create_dir_all(coff_file.parent().unwrap())?;
			}

			let mut res_compiler_args = Vec::from([
				"-v".to_string(),
				resource_file.to_str().unwrap_or("ERROR").to_string(),
				"-OCOFF".to_string(),
			]);

			for incl in project.include_paths.clone() {
				res_compiler_args.push(format!("-I{incl}"));
			}

			for define in project.defines.clone() {
				res_compiler_args.push(format!("-D{define}"));
			}

			res_compiler_args
				.push(format!("-o{}", coff_file.to_str().unwrap()));

			self.system.execute(
				resource_compiler
					.args(&res_compiler_args)
					.current_dir(&(self.environment).project_directory),
			)?;

			o_files.push(coff_file.to_str().unwrap().to_string());
		}

		self.ui.remove_bar(progress);

		Ok(())
	}

	fn linking_step(
		&mut self,
		project: &Project,
		out_dir: &PathBuf,
		mingw: &String,
		output: &String,
		o_files: &mut Vec<String>,
	) -> anyhow::Result<()> {
		let spinner = self.ui.create_spinner("Linking...");
		match project.project_type {
			ProjectType::StaticLibrary => {
				let mut linker = Command::new(mingw.clone() + "ar");
				let mut linker_args = Vec::from([
					"rcs".to_string(),
					format!(
						"{}/{}",
						&out_dir.to_str().unwrap_or("ERROR"),
						output
					),
				]);

				linker_args.append(o_files);

				for def_file in
					project.source_files.get(&SourceFileType::ModuleDefinition)
				{
					linker_args.push(def_file.to_str().unwrap().to_string());
				}

				self.system.execute(
					linker
						.args(&linker_args)
						.current_dir(&(self.environment).project_directory),
				)?;
			}

			_ => {
				let mut linker = Command::new(
					mingw.clone()
						+ match project.language {
							ProjectLanguage::C => "gcc",
							ProjectLanguage::CPP => "g++",
						},
				);
				let mut linker_args = Vec::new();

				linker_args.append(o_files);

				for def_file in
					project.source_files.get(&SourceFileType::ModuleDefinition)
				{
					linker_args.push(def_file.to_str().unwrap().to_string());
				}

				for path in project.lib_paths.clone() {
					linker_args.push(format!("-L{path}"))
				}

				for lib in project.libs.clone() {
					linker_args.push(format!("-l{lib}"))
				}

				for flag in project.compiler_flags.clone() {
					linker_args.push(flag)
				}

				for flag in project.linker_flags.clone() {
					linker_args.push("-Wl,".to_string() + &flag)
				}

				linker_args.push(format!(
					"-o{}/{}",
					&out_dir.to_str().unwrap_or("ERROR"),
					output
				));

				self.system.execute(
					linker
						.args(&linker_args)
						.current_dir(&(self.environment).project_directory),
				)?;
			}
		}

		self.ui.remove_bar(spinner);

		Ok(())
	}

	fn build(
		&mut self,
		project: &Project,
	) -> anyhow::Result<()> {
		let obj_dir: PathBuf = (self.environment)
			.numake_directory
			.join(format!("obj/{}", &project.name));
		let out_dir: PathBuf = (self.environment)
			.numake_directory
			.join(format!("out/{}", &project.name));

		let res_dir: PathBuf = (self.environment)
			.numake_directory
			.join(format!("res/{}", &project.name));

		if !obj_dir.exists() {
			fs::create_dir_all(&obj_dir)?;
		}

		if !out_dir.exists() {
			fs::create_dir_all(&out_dir)?;
		}

		if !res_dir.exists() {
			fs::create_dir_all(&res_dir)?;
		}

		let mut o_files: Vec<String> = Vec::new(); // Can't assume all compilers support wildcards.

		let mingw = format!(
			"{}-w64-mingw32-",
			project.arch.clone().unwrap_or("x86_64".to_string())
		);

		self.compile_step(project, &obj_dir, &mingw, &mut o_files)?;
		self.resource_step(project, &mingw, &res_dir, &mut o_files)?;
		self.linking_step(
			project,
			&out_dir,
			&mingw,
			&project.output.clone().unwrap_or("out".to_string()),
			&mut o_files,
		)?;

		project.copy_assets(&self.environment.project_directory, &out_dir)?;

		Ok(())
	}
}

impl UserData for MinGW {
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
		methods.add_method_mut("build", |_, this, project: Project| match this
			.build(&project)
		{
			Ok(_) => Ok(()),
			Err(err) => Err(mlua::Error::external(err)),
		})
	}
}

impl FromLua for MinGW {
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

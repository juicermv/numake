use std::{
	collections::{
		HashMap,
		HashSet,
	},
	fs,
	path::{
		Path,
		PathBuf,
	},
	process::Command,
};

use mlua::{
	UserData,
	UserDataMethods,
};
use pathdiff::diff_paths;

use crate::lib::{
	data::{
		environment::Environment,
		project::Project,
		source_file_type::SourceFileType,
	},
	runtime::system::System,
	ui::UI,
	util::build_cache::BuildCache,
};

#[derive(Clone)]
pub struct Generic
{
	environment: Environment,
	cache: BuildCache,
	ui: UI,
	system: System,
}

impl Generic
{
	pub fn new(
		environment: Environment,
		cache: BuildCache,
		ui: UI,
		system: System,
	) -> Self
	{
		Generic {
			environment,
			cache,
			ui,
			system,
		}
	}

	fn compile_step(
		&mut self,
		project: &Project,
		toolset_compiler: &String,
		obj_dir: &PathBuf,
		o_files: &mut Vec<String>,
	) -> anyhow::Result<()>
	{
		let source_files = project.source_files.get(&SourceFileType::Code);

		let mut generic_cache: HashSet<String> = self
			.cache
			.read_set(&(toolset_compiler.to_string() + "_cache"))?;

		let hashes: HashMap<&PathBuf, String> = source_files
			.iter()
			.filter_map(|file| {
				match sha256::try_digest(file) {
					Ok(digest) => Some((file, digest)),
					Err(_) => None,
				}
			})
			.collect();

		let dirty_files: Vec<&PathBuf> = source_files
			.iter()
			.filter(|file| {
				match hashes.get(file) {
					Some(hash) => !generic_cache.contains(hash),

					None => true,
				}
			})
			.collect();

		let clean_hashes: HashSet<String> = source_files
			.iter()
			.filter_map(|file| {
				match hashes.get(file) {
					Some(hash) => {
						if generic_cache.contains(hash) {
							Some(hash.clone())
						} else {
							None
						}
					}

					None => None,
				}
			})
			.collect();

		let progress = self
			.ui
			.create_bar(dirty_files.len() as u64, "Compiling... ");

		generic_cache = clean_hashes;

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
			let mut compiler = Command::new(toolset_compiler);

			if !o_file.parent().unwrap().exists() {
				fs::create_dir_all(o_file.parent().unwrap())?;
			}

			let mut compiler_args = Vec::from([
				"-c".to_string(),
				format!("-o{}", o_file.to_str().unwrap()),
			]);

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
					generic_cache.insert(hashes[&file].clone());
					Ok(status)
				}

				Err(err) => Err(err),
			}?;
		}

		self.ui.remove_bar(progress);

		self.cache.write_set(
			&(toolset_compiler.to_string() + "_cache"),
			generic_cache,
		)?;

		Ok(())
	}

	fn linking_step(
		&mut self,
		project: &Project,
		toolset_linker: &String,
		out_dir: &Path,
		output: &String,
		o_files: Vec<String>,
	) -> anyhow::Result<()>
	{
		let spinner = self.ui.create_spinner("Linking...");
		let mut linker = Command::new(toolset_linker);
		let mut linker_args = Vec::new();

		linker_args.append(
			&mut o_files
				.iter()
				.filter_map(|absolute_path| {
					Some(
						diff_paths(
							absolute_path,
							&(self.environment).project_directory,
						)?
						.to_str()?
						.to_string(),
					)
				})
				.collect(),
		);

		for path in project.lib_paths.clone() {
			linker_args.push(format!("-L{path}"))
		}

		for lib in project.libs.clone() {
			linker_args.push(format!("-l{lib}"))
		}

		for flag in project.linker_flags.clone() {
			linker_args.push(flag)
		}

		linker_args.push(format!(
			"-o{}/{}",
			&out_dir.to_str().unwrap_or("ERROR"),
			&output
		));

		self.system.execute(
			linker
				.args(&linker_args)
				.current_dir(&(self.environment).project_directory),
		)?;

		self.ui.remove_bar(spinner);

		Ok(())
	}

	fn build(
		&mut self,
		toolset_compiler: &String,
		toolset_linker: &String,
		project: &Project,
	) -> anyhow::Result<()>
	{
		let obj_dir: PathBuf = (self.environment)
			.numake_directory
			.join(format!("obj/{}", project.name));
		let out_dir: PathBuf = (self.environment)
			.numake_directory
			.join(format!("out/{}", project.name));

		if !obj_dir.exists() {
			fs::create_dir_all(&obj_dir)?;
		}

		if !out_dir.exists() {
			fs::create_dir_all(&out_dir)?;
		}

		let mut o_files: Vec<String> = Vec::new(); // Can't assume all compilers support wildcards.

		self.compile_step(project, toolset_compiler, &obj_dir, &mut o_files)?;
		self.linking_step(
			project,
			toolset_linker,
			&out_dir,
			&project.output.clone().unwrap_or("out".to_string()),
			o_files,
		)?;

		project.copy_assets(&self.environment.project_directory, &out_dir)?;

		Ok(())
	}
}

impl UserData for Generic
{
	fn add_methods<M: UserDataMethods<Self>>(methods: &mut M)
	{
		methods.add_method_mut(
			"build",
			|_,
			 this,
			 (project, compiler, linker): (Project, String, String)| {
				match this.build(&compiler, &linker, &project) {
					Ok(_) => Ok(()),
					Err(err) => Err(mlua::Error::external(err)),
				}
			},
		)
	}
}

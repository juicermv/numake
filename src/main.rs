// @formatter:on

/*
    TODO: Optimization, Refactoring, Error Handling. THIS IS A WIP!
*/

mod project;

use clap::{Parser};
use mlua::{Lua, Table};
use std::{fs};


use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use tempfile::tempfile;
use uuid::{Uuid};
use zip::ZipArchive;
use crate::project::PROJECT;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(long, default_value = "unspecified")]
    target: String,

    #[arg(long, default_value= "unspecified")]
    configuration: String,

    #[arg(long, default_value = "unspecified")]
    arch: String,

    #[arg(long, default_value = "unspecified")]
    toolset_compiler: String,

    #[arg(long, default_value = "unspecified")]
    toolset_linker: String,

    #[arg(long, default_value = "PROJECT.numake")]
    file: String,

    #[arg(long, default_value = "out")]
    output: String,

    #[arg(long, default_value = ".")]
    workdir: String,

    #[arg(long)]
    msvc: bool
}


fn main() {
    let now = SystemTime::now();

    let args: Args = Args::parse();

    // Will handle obj, build output as well as includes and libs in this directory.
    if fs::metadata(format!("{}/.numake", &args.workdir)).is_err() {
        fs::create_dir( format!("{}/.numake", &args.workdir)).unwrap(); // Create workdir if it does not exist
    }
    let workdir: PathBuf = dunce::canonicalize(args.workdir.to_string() + "/.numake").unwrap();

    unsafe {
        PROJECT.workdir = workdir.to_str().unwrap().to_string();
    }

    // Setup Lua
    let lua: Lua = Lua::new();
    let globals: Table = lua.globals();

    // Environment vars
    globals.set("msvc", args.msvc).unwrap();
    globals.set("target", args.target).unwrap();
    globals.set("configuration", args.configuration).unwrap();
    globals.set("arch", args.arch).unwrap();
    globals.set("toolset_compiler", args.toolset_compiler).unwrap();
    globals.set("toolset_linker", args.toolset_linker).unwrap();
    globals.set("output", args.output).unwrap();

    // Functions
    globals.set("add_include_path", lua.create_function(
        |_, path:String| unsafe {
            PROJECT.add_include_path(path)
        }
    ).unwrap()).unwrap();

    globals.set("set_include_paths", lua.create_function(
        |_, paths:Vec<String>| unsafe {
            PROJECT.set_include_paths(paths)
        }
    ).unwrap()).unwrap();

    globals.set("add_compiler_flag", lua.create_function(
        |_, flag:String| unsafe {
            PROJECT.add_compiler_flag(flag)
        }
    ).unwrap()).unwrap();

    globals.set("set_compiler_flags", lua.create_function(
        |_, flags:Vec<String>| unsafe {
            PROJECT.set_compiler_flags(flags)
        }
    ).unwrap()).unwrap();

    globals.set("add_linker_flag", lua.create_function(
        |_, flag:String| unsafe {
            PROJECT.add_linker_flag(flag)
        }
    ).unwrap()).unwrap();

    globals.set("set_linker_flags", lua.create_function(
        |_, flags:Vec<String>| unsafe {
            PROJECT.set_linker_flags(flags)
        }
    ).unwrap()).unwrap();

    globals.set("add_lib_path", lua.create_function(
        |_, path:String| unsafe {
            PROJECT.add_lib_path(path)
        }
    ).unwrap()).unwrap();

    globals.set("set_lib_paths", lua.create_function(
        |_, paths:Vec<String>| unsafe {
            PROJECT.set_lib_paths(paths)
        }
    ).unwrap()).unwrap();

    globals.set("add_lib", lua.create_function(
        |_, lib:String| unsafe {
            PROJECT.add_lib(lib)
        }
    ).unwrap()).unwrap();

    globals.set("set_libs", lua.create_function(
        |_, libs:Vec<String>| unsafe {
            PROJECT.set_libs(libs)
        }
    ).unwrap()).unwrap();

    globals.set("add_dir", lua.create_function(
        |_, (path, recursive):(String, bool)| unsafe {
            PROJECT.add_dir(&Path::new(&PROJECT.workdir).parent().unwrap().join(path), recursive)
        }
    ).unwrap()).unwrap();

    globals.set("add_file", lua.create_function(
        |_, path:String| unsafe {
            PROJECT.add_file(&Path::new(&PROJECT.workdir).parent().unwrap().join(path))
        }
    ).unwrap()).unwrap();

    globals.set("add_asset", lua.create_function(
        |_, (filepath, newpath):(String, String)| unsafe {
            PROJECT.add_asset(filepath, newpath)
        }
    ).unwrap()).unwrap();

    globals.set("define", lua.create_function(
        |_, define:String| unsafe {
            PROJECT.define(define)
        }
    ).unwrap()).unwrap();

    // Not used but here for future reference
    /*
    globals.set("get_workspace", lua.create_function(
        |_| {
            Ok(
                workdir.to_str().unwrap()
            )
        }
    ).unwrap()).unwrap();


    globals.set("workspace_mkdir", lua.create_function(
        |_, (path):(String)|{
            let path_buf = dunce::canonicalize(path).unwrap();
            if(path_buf.starts_with(workdir.as_path())){
                if(!path_buf.exists()){
                    fs::create_dir_all(path_buf).unwrap();
                }
                Ok(0)
            } else {
                Err("Tried to create directory outside workspace!").unwrap()
            }
        }
    ).unwrap()).unwrap();
     */


    // Downloads and extracts a zip in the workspace, for downloading external assets and resources.
    // Returns path of extracted contents
    globals.set("workspace_download_zip", lua.create_function(
        |_, url:String| unsafe {
            let response = reqwest::blocking::get(&url).unwrap();

            if response.status().is_success() {
                // Where the archive will be extracted.
                let buf: [u8; 16] = *url.as_bytes().last_chunk::<16>().unwrap();
                let path = format!("{}/remote/{}", PROJECT.workdir,Uuid::new_v8(buf));
                if fs::metadata(&path).is_err() { // Don't "download" again. (data already in memory)
                    fs::create_dir_all(&path).unwrap();
                    let path_buf = dunce::canonicalize(&path).unwrap();
                    let mut tempfile = tempfile()?; // Create a tempfile as a buffer for our response bytes because nothing else implements Seek ffs
                    tempfile.write_all(response.bytes().unwrap().as_ref())?;

                    let mut zip = ZipArchive::new(BufReader::new(tempfile)).unwrap();
                    zip.extract(&path_buf).unwrap();
                }
                Ok(path)
            } else {
                Ok(response.status().to_string())
            }
        }
    ).unwrap()).unwrap();

    // Get and require an online numake script, for package management mainly.
    globals.set("require_url", lua.create_function(
        |lua, url:String| {
            let response = reqwest::blocking::get(url).unwrap();
            lua.load(response.text().unwrap().to_string()).exec().unwrap();
            Ok(0)
        }
    ).unwrap()).unwrap();

    // Parse file
    lua.load(fs::read_to_string(Path::new(&args.workdir).join(&args.file)).unwrap()).exec().unwrap();


    // NO! YOU MAY NOT MODIFY STATIC VALUES! IT IS UNSAFE!!! :O
    unsafe {
        // Due to MLua borrowing values and not giving them back :<
        let toolset_compiler: String = globals.get("toolset_compiler").unwrap();
        let toolset_linker: String = globals.get("toolset_linker").unwrap();
        let target: String = globals.get("target").unwrap();
        let arch: String = globals.get("arch").unwrap();
        let configuration: String = globals.get("configuration").unwrap();
        let output: String = globals.get("output").unwrap();
        let msvc: bool = globals.get("msvc").unwrap();


        let config: String = format!("{}-{}-{}", &arch, &target, &configuration);

        // Do we really need to throw an error if the folder exists, and we try to create it again? @RustDevs 0_0
        if !workdir.join(format!("obj/{}", &config)).exists() {
            fs::create_dir_all(workdir.join(format!("obj/{}", &config))).unwrap();
        }

        if !workdir.join(format!("out/{}", &config)).exists() {
            fs::create_dir_all(workdir.join(format!("out/{}", &config))).unwrap();
        }

        let obj_dir: String = dunce::canonicalize(workdir.join(format!("obj/{}", &config))).unwrap().to_str().unwrap().to_string();
        let out_dir: String = dunce::canonicalize(workdir.join(format!("out/{}", &config))).unwrap().to_str().unwrap().to_string();

        let mut o_files: String = "".to_string(); // Can't assume all compilers support wildcards.

        for file in PROJECT.files.to_vec() {
            let mut compiler = Command::new(&toolset_compiler);

            let o_file = format!("{}/{}.{}", &obj_dir, file.file_name().unwrap().to_str().unwrap(), if msvc {"obj"} else {"o"});

            let mut compiler_args = Vec::from([
                "-c".to_string(),
                file.to_str().unwrap().to_string(),
                if msvc {format!("-Fo:{}", &o_file)} else{format!("-o{}", &o_file)},
            ]);

            o_files += &format!("{} ", &o_file);

            for incl in PROJECT.include_paths.to_vec() {
                compiler_args.push(
                    format!("-I{incl}")
                )
            }

            for define in PROJECT.defines.to_vec() {
                compiler_args.push(
                    format!("-D{define}")
                )
            }

            for flag in PROJECT.compiler_flags.to_vec() {
                compiler_args.push(
                    flag
                )
            }

            println!("{}", &args.workdir);
            println!("{} exited with {}.",
                &toolset_compiler,
                compiler.args(&compiler_args).current_dir(dunce::canonicalize(&args.workdir).unwrap()).status().unwrap()
            );
        }

        let mut linker = Command::new(&toolset_linker);
        let mut linker_args =
            Vec::from([
                format!("{}", o_files)
            ]);


        for lib in PROJECT.libs.to_vec() {
            linker_args.push(
                if msvc { lib } else {format!("-l{lib}")}
            )
        }

        if msvc {
            linker_args.push("/link".to_string());
        }

        if msvc {
            linker_args.push(format!("/out:{}/{}", out_dir.to_string(), output.to_string()));
        } else {
            linker_args.push(format!("-o{}/{}", out_dir.to_string(), output.to_string()));
        }

        for flag in PROJECT.linker_flags.to_vec() {
            linker_args.push(
                flag
            )
        }

        for path in PROJECT.lib_paths.to_vec() {
            linker_args.push(
                if msvc {format!("/LIBPATH:{path}")} else {format!("-L{path}")}
            )
        }

        println!("{} exited with {}. ",
                 &toolset_linker,
                 linker.args(&linker_args[0..linker_args.len()-1]).current_dir(dunce::canonicalize(&args.workdir).unwrap()).status().unwrap()
        );

        for (oldpath, newpath) in PROJECT.assets.to_vec() {
            let old_path = Path::new(&oldpath); // Already canonicalized in project.rs
            let new_path = Path::new(&out_dir).join(Path::new(&newpath));

            if old_path.starts_with(dunce::canonicalize(&args.workdir).unwrap()) {
                if new_path.starts_with(&out_dir) { // Make sure we haven't escaped our output dir
                    fs::copy(old_path, new_path).unwrap();
                } else {
                    Err(format!("Asset file '{}' copied to invalid destination! ({})", old_path.to_str().unwrap(), new_path.to_str().unwrap())).unwrap()
                }
            } else {
                Err(format!("Asset file '{}' originates outside workspace!", old_path.to_str().unwrap())).unwrap()
            }
        }
    }
    println!("\n\nNuMake done in {}ms!",now.elapsed().unwrap().as_millis());
}

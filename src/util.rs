use std::{
	collections::HashMap,
	fs,
	path::Path,
	process::Command,
};

use anyhow::anyhow;
use base32::Alphabet;
use mlua::{
	Integer,
	IntoLua,
	Lua,
};

pub fn hash_string(val: &String) -> String { base32::encode(Alphabet::Crockford, val.as_bytes()) }

// Horror
pub fn into_lua_value<'lua>(
	lua: &'lua Lua,
	origin: &toml::Value,
) -> mlua::Result<mlua::Value<'lua>>
{
	let mut dest: mlua::Value = mlua::Value::Nil;
	if origin.is_bool() {
		dest = mlua::Value::Boolean(origin.as_bool().unwrap())
	} else if origin.is_array() {
		dest = origin
			.as_array()
			.unwrap()
			.iter()
			.map(|val| into_lua_value(lua, val).unwrap_or(mlua::Nil))
			.collect::<Vec<mlua::Value>>()
			.into_lua(lua)?;
	} else if origin.is_table() {
		dest = mlua::Value::Table(into_lua_table(
			lua,
			origin.as_table().unwrap(),
		)?);
	} else if origin.is_float() {
		dest = mlua::Value::Number(origin.as_float().unwrap());
	} else if origin.is_str() {
		dest =
			mlua::Value::String(lua.create_string(origin.as_str().unwrap())?);
	} else if origin.is_integer() {
		dest = mlua::Value::Integer(origin.as_integer().unwrap() as Integer);
	} else if origin.is_datetime() {
		dest = mlua::Value::String(
			lua.create_string(origin.as_datetime().unwrap().to_string())?,
		);
	}

	Ok(dest)
}

pub fn into_lua_table<'lua>(
	lua: &'lua Lua,
	origin: &toml::Table,
) -> mlua::Result<mlua::Table<'lua>>
{
	let dest: mlua::Table = lua.create_table()?;
	for (key, val) in origin {
		dest.set(key.clone(), into_lua_value(lua, val)?)?;
	}

	Ok(dest)
}

pub fn into_toml_value(origin: &mlua::Value) -> mlua::Result<toml::Value>
{
	let mut dest = toml::Value::String("Nil".to_string());
	if origin.is_table() {
		dest = toml::Value::Table(into_toml_table(origin.as_table().unwrap())?);
	} else if origin.is_integer() {
		dest = toml::Value::Integer(origin.as_i64().unwrap());
	} else if origin.is_boolean() {
		dest = toml::Value::Boolean(origin.as_boolean().unwrap());
	} else if origin.is_number() {
		dest = toml::Value::Float(origin.as_number().unwrap());
	} else if origin.is_string() {
		dest = toml::Value::String(origin.as_str().unwrap().to_string());
	}
	Ok(dest)
}

pub fn into_toml_table(origin: &mlua::Table) -> mlua::Result<toml::Table>
{
	let mut dest = toml::Table::new();
	// Love this
	for pair in origin.clone().pairs::<String, mlua::Value>() {
		let (key, val) = pair?;
		dest.insert(key, into_toml_value(&val)?);
	}
	Ok(dest)
}

pub fn to_lua_result<T>(val: anyhow::Result<T>) -> mlua::Result<T>
{
	if val.is_err() {
		Err(mlua::Error::external(val.err().unwrap()))?
	} else {
		Ok(val.ok().unwrap())
	}
}

pub fn download_vswhere<P: AsRef<Path>>(path: &P) -> anyhow::Result<()>
{
	let response = reqwest::blocking::get("https://github.com/microsoft/vswhere/releases/latest/download/vswhere.exe")?;
	if response.status().is_success() {
		fs::write(path, response.bytes()?.as_ref())?;
		Ok(())
	} else {
		Err(anyhow!(response.status()))
	}
}

pub fn args_to_map(args: Vec<String>) -> HashMap<String, Option<String>>
{
	let mut output: HashMap<String, Option<String>> = HashMap::new();

	for arg in args {
		let split_arg = arg
			.split("=")
			.map(|val| val.to_string())
			.collect::<Vec<String>>();

		if split_arg.len() == 2 {
			output.insert(split_arg[0].clone(), Some(split_arg[1].clone()));
		} else if split_arg.len() > 1 {
			output.insert(
				split_arg[0].clone(),
				Some(
					split_arg[1 ..]
						.iter()
						.map(|val| val.clone() + "=")
						.collect::<String>(),
				),
			);
		} else {
			output.insert(split_arg[0].clone(), None);
		}
	}

	output
}

pub fn get_gcc_includes(cmd: String) -> anyhow::Result<Vec<String>>
{
	let result = Command::new(&cmd).args(["-E", "-v", "-"]).output();
	if result.is_ok() {
		let mut return_vec: Vec<String> = Vec::new();
		let mut output =
			String::from_utf8_lossy(result.ok().unwrap().stderr.as_slice())
				.to_string();
		output = output
			.split("#include <...> search starts here:")
			.collect::<Vec<&str>>()[1]
			.to_string();
		output = output.split("End of search list.").collect::<Vec<&str>>()[0]
			.to_string();
		for line_r in output.lines() {
			let mut line = line_r.to_string();
			if line.is_empty() {
				continue;
			}

			while line.starts_with(' ') {
				line.replace_range(0..1, "");
			}

			line = line.replace(" /", "/");

			if line.ends_with(" (framework directory)") {
				// MacOS specific
				return_vec.push(
					dunce::canonicalize(
						line.replace(" (framework directory)", "")
					)?
					.to_str()
					.unwrap()
					.to_string(),
				);
			} else {
				return_vec.push(
					dunce::canonicalize(line)?
						.to_str()
						.unwrap()
						.to_string(),
				);
			}
		}
		Ok(return_vec)
	} else {
		Ok(Vec::default())
	}
}

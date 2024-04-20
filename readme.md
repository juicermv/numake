> [!WARNING]
> THIS IS A WIP AND IN NO WAY PRODUCTION READY! THERE'S NO ERROR HANDLING!

# NuMake is a small, portable, and easy-to-use cross-platform C/C++ build system!

<b>IT IS NOT</b> a code generation tool like CMake, PreMake, etc.
NuMake does not take your code and convert it into any other type of Makefile or Visual Studio Solution.

Instead, NuMake talks directly to the compiler. Whether it is MSVC, GCC, or Clang.

NuMake utilizes [Luau](https://github.com/Roblox/Luau), a sandboxed version of Lua originally built by Roblox. It does not write anything to your filesystem outside the `.numake` folder (and the good ol temp file here and there ;)).

This ensures that any NuMake scripts you run on your system will not affect anything on it.

## Usage

First, build NuMake using `cargo build` so you can actually start using it!

### Terminology
#### `workspace`
Your `workspace` is your local `.numake` directory.
<br>
This directory will contain generated object files, binaries, and downloaded things!

#### NuMake script
This is your main `*.numake` file.
By default, the NuMake binary will try to read `project.numake`, but by passing `--file <name>` you can make NuMake try to read any file you want. 

### Datatypes and arguments of the sort
NuMake is mainly data-oriented, meaning there's no OOP-style objects.
<br>
There are no objects to describe a project, a target, or anything of the sort.
There are three strings that can be passed through the command line, and with which you can do pretty much anything you want:

#### `arch`:
Passed via `--arch <val>`. 
<br>
Again, this just a string. If you want to define a limited set of architectures you want your project to compile to, just do so in the script with Lua's logic.

#### `target`:
Passed via `--target <val>`. 
<br>
Just like `arch`, this is just a string. Do with it as you please.

#### `configuration`:
Passed via `--configuration <val>`. 
<br>
You're not going to believe this... same deal.

### Well, this is underwhelming.
Not really though. What makes NuMake great is its simplicity and flexibility. It's more readable than your usual Makefile and shell script, and adds some _neat features!_

### Neat features
#### Toolchain Compatibility
NuMake supports GCC, Clang, and MSVC.
You can use the same codebase and same NuMake file for any platform, you don't need a seperate code generation step.

MSVC (or more specifically, cl) uses very different arguments than GCC or Clang for even the most basic things like linker arguments, or telling the compiler where to output object files. For this reason NuMake has a simple bool, `msvc`, that you can set to true in order to signal to NuMake you're using MSVC.

#### Package Management (kinda)
Mostly for Windows, where we don't have a universal package manager for both libraries and programs :'(

In your NuMake script, you can use `workspace_download_zip(url)` with a url to a zip file, in order to download and extract it in your local `workspace` under the `remote` directory. The function itself will return a String with the extracted contents' absolute path, so you can access it in the rest of your script.

#### Remote scripts
Since Luau is completely sandboxed, and has no access to your machine's filesystem outside of what you allow it to access, we have the `require_url(url)` function which will try to read, load, and execute the script at the given URL. This can be useful for various utilities.
<br>
For example, you could have a utility script that would automatically download some library, and link it to your project, which essentially automates a process that could be otherwise very frustrating.
And due to the script running as part of *your* NuMake script, it can already change and modify what it does basically on your variables.
<br>
So let's say you have the `msvc` bool set, the script could take this into account, and automatically download and link the `msvc` distribution of the hypothetical library and vice versa. The possibilities are limitless.

## In Action
You can look at and try the example(s) under _`examples`_ in this repository.

## Reference
At the moment, NuMake doesn't have much, so I might as well include it here.

### Variables
#### `msvc`: Bool
Internally, changes the arguments passed to the compiler and linker.
This is false by default and can be set either by just setting it to true in your script or feeding it via the commandline with `--msvc`.

#### `arch`, `target`, `configuration`: String
Like `msvc`, you can either set these in the script itself or feed them through the commandline like shown earlier. These are all set to `"unspecified"` by default, and they are actually used to name the directory under `workspace/out` where your output is stored.

#### `workdir`: String
This one can only be fed through the commandline via `--workdir <path>` and is not accessible to your script. This changes the working directory of NuMake itself. So let's say you have NuMake in one place and want to run a script in another directory, just pass where your script is located to the `workdir` and it'll run it. Do note that the source code itself needs to be in the same directory as the script. Child directories are fine too.

#### `output`: String
This sets the name of whatever it is you're building. So a DLL would have its `output` be `"adll.dll"` same goes for an exe or static library or whatever else you're doing! This is completely accessible via the script and can be fed through the commandline like the other variables.

#### `toolset_compiler`, `toolset_linker`: String
These are the paths to your compiler and linker respectively. Can be fully modified in your script as well as passed through the commandline.


#### `file`: String
If you for some reason feel like running a script that isn't named `project.numake` you can feed this to the program to make it do so. This is not accessible through the script itself.

### Script Functions

#### `add_include_path( path: String )`
Adds the `path` to the include paths list that is eventually passed to the compiler.

#### `set_include_paths( paths: { String } )`
Takes in an array of strings which will be used as the include path list that will be passed to the compiler. This will overwrite the current include path list.

#### `add_lib_path( path: String )`
Adds the `path` to the library paths list that is eventually passed to the linker.

#### `set_lib_paths( paths: { String } )`
Takes in an array of strings which will be used as the library path list that will be passed to the linker. This will overwrite the current linker path list.

#### `add_lib( name: String)`
Adds a library to the list of libraries that will be linked against your code.

#### `set_libs( libs: {String } )`
Sets the list of libraries that will be linked against your code. This will overwrite the list.

#### `add_compiler_flag( name: String)`
Adds a flag to be passed to the compiler on compilation. Keep in mind that if you're passing an argument that takes a spaced paramater you need to pass the parameter in a seperate call.

#### `set_compiler_flags( flags: { String })`
Sets the flags passed to the compiler. Will overwrite anything added previously.

#### `add_linker_flag( name: String)`
Adds a flag to be passed to the linker. Keep in mind that if you're passing an argument that takes a spaced paramater you need to pass the parameter in a seperate call.

#### `set_linker_flags( flags: { String })`
Sets the flags passed to the linker. Will overwrite anything added previously.

#### `add_dir(path: String, recursive: bool)`
Adds files in the specified directory for compilation. Optionally goes over subdirectories as well.

#### `add_file(path: String)`
Adds source file to be compiled.

#### `add_asset(path: String, newpath: String)`
If you have any files that you need with your program this will take them and copy them to the output directory. `newpath` must be relative.

#### `define( val: String )`
Adds a preprocessor definition.

#### `workspace_download_zip( url: String )`
Downloads the specified zip and extracts it to a uniquely named folder under `workspace/remote`. Returns the path of the extracted contents as an asbolute path.

#### `require_url( url: String )`
Loads and executes script from given url.
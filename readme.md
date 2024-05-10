> [!WARNING]
> THIS IS A WIP!

# NuMake is a small, portable, and easy-to-use cross-platform C/C++ build system!

<b>IT IS NOT</b> a code generation tool like CMake, PreMake, etc.
NuMake does not take your code and convert it into any other type of Makefile or Visual Studio Solution.

Instead, NuMake talks directly to the compiler. Whether it is MSVC, GCC, or Clang.

NuMake utilizes [Luau](https://github.com/Roblox/Luau), a sandboxed version of Lua originally built by Roblox. It does not write anything to your filesystem outside the `.numake` folder (and the good ol temp file here and there ;) ).
<br>This ensures that any NuMake scripts you run on your system will not affect anything on it.

## Usage

First, build NuMake using `cargo build` so you can actually start using it! 
This will output the executable you can pass arguments to!
Now continue on to the wiki! (TBD)
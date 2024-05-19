> [!WARNING]
> THIS IS A WIP!

# NuMake is a small, portable, and easy-to-use cross-platform C/C++ build system!

<b>IT IS NOT</b> a code generation tool like CMake, PreMake, etc.
NuMake does not take your code and convert it into any other type of Makefile or Visual Studio Solution.

Instead, NuMake talks directly to the compiler. Whether it is MSVC, GCC, or Clang.

NuMake utilizes [Luau,](https://github.com/Roblox/Luau) a sandboxed version of Lua originally built by Roblox. It does not write anything to your filesystem outside the `.numake` folder (and the good ol temp file here and there ;) ).
<br>This ensures that any NuMake scripts you run on your system will not affect anything on it.

## Usage

First, either build numake itself or grab a binary from the releases. You can check out examples for examples of how the Lua stuff works. You can run `numake help` to get help for the command-line executable. Make sure to also check out the [wiki.](https://github.com/juicermv/numake/wiki)
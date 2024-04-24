workspace:require_url("https://pastebin.com/raw/rXKD6pjn") -- Just a test, doesn't actually do anything.

sdl_path = workspace:download_zip("https://github.com/libsdl-org/SDL/releases/download/release-2.30.2/SDL2-devel-2.30.2-mingw.zip") .. "/SDL2-2.30.2/x86_64-w64-mingw32"
mingw = workspace:create_target("mingw")

mingw.compiler = "x86_64-w64-mingw32-g++"
mingw.linker = mingw.compiler

mingw.output = "test.exe"
mingw.include_paths = {sdl_path .. "/include"}
mingw.library_paths = {sdl_path .. "/lib"}
mingw.libraries = {
    "mingw32",
    "SDL2",
    "SDL2main",
    "m",
    "dinput8",
    "dxguid",
    "dxerr8",
    "user32",
    "gdi32",
    "winmm",
    "imm32",
    "ole32",
    "oleaut32",
    "shell32",
    "setupapi",
    "version",
    "uuid"
}

mingw.compiler_flags = {"--verbose", "-mwindows", "-Wl,--nxcompat", "-Wl,--high-entropy-va", "-static", "-Wl,-Bstatic"}
mingw.linker_flags = mingw.compiler_flags
mingw.assets = { [sdl_path .. "/bin/SDL2.dll"] = "SDL2.dll" }
mingw:add_dir("src")

gcc = workspace:create_target("gcc")
gcc.output = "test"
gcc.libraries = {
    "SDL2"
}

gcc:add_dir("src")

workspace:register_target(gcc)
workspace:register_target(mingw)
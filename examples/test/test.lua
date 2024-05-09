inspect = workspace:require_url("https://raw.githubusercontent.com/kikito/inspect.lua/master/inspect.lua")

if workspace:get("test") == nil then
    workspace:set("test", 1234)
end

sdl_path = workspace:download_zip("https://github.com/libsdl-org/SDL/releases/download/release-2.30.2/SDL2-devel-2.30.2-mingw.zip") .. "/SDL2-2.30.2/x86_64-w64-mingw32"
mingw = workspace:create_target("mingw")

mingw.compiler = "x86_64-w64-mingw32-g++"
mingw.linker = mingw.compiler

mingw.output = "test.exe"
mingw.include_paths = {sdl_path .. "/include"}
mingw.library_paths = {sdl_path .. "/lib"}
mingw.libraries = {
    "mingw32",
    "SDL2.dll",
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
mingw.files = workspace:walk_dir("src")

sdl_path_msvc = workspace:download_zip("https:/github.com/libsdl-org/SDL/releases/download/release-2.30.2/SDL2-devel-2.30.2-VC.zip")

msvc = workspace:create_msvc_target("msvc")
msvc.include_paths = {
    sdl_path_msvc .. "/SDL2-2.30.2/include"
}

msvc.libraries = {
    "shell32.lib",
    "SDL2.lib",
    "SDL2main.lib"
}

msvc.library_paths = {
    sdl_path_msvc .. "/SDL2-2.30.2/lib/x64",
}

msvc.compiler_flags = { "/EHsc" }
msvc.linker_flags = { "/SUBSYSTEM:WINDOWS" }
msvc.output = "test.exe"
msvc.definitions = { "MSVC" }
msvc.files = workspace:walk_dir("src")
msvc.arch = "x64"



gcc = workspace:create_target("gcc")
gcc.output = "test"
gcc.libraries = {
    "SDL2"
}
gcc.compiler = "g++"
gcc.linker = gcc.compiler

gcc.files = workspace:walk_dir("src")

workspace:register_target(gcc)
workspace:register_target(mingw)
workspace:register_target(msvc)

print(workspace:get("test"))
print(inspect(workspace:env()))
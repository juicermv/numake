--- Download SDL2 for MSVC and mingw
sdl_path_mingw = workspace:download_zip("https://github.com/libsdl-org/SDL/releases/download/release-2.30.2/SDL2-devel-2.30.2-mingw.zip") .. "/SDL2-2.30.2"
sdl_path_msvc = workspace:download_zip("https:/github.com/libsdl-org/SDL/releases/download/release-2.30.2/SDL2-devel-2.30.2-VC.zip")

--- You can use the cache system to save user-specific preferences and things of the sort.
--- You can store almost every lua variable type but for this example we'll use a string:
if workspace:get("ice_cream") == nil then
    ice_cream = workspace.arguments["ice_cream"] --- Read user-specificed arguments
    workspace:set("ice_cream", ice_cream)
else
    print("I know your favorite ice cream flavor! It's " .. workspace:get("ice_cream") .. "!")
end

--- MSVC 64 BIT TARGET
msvc = workspace:create_msvc_target("msvc") --- We create targets via the workspace so they can inherit some important values!
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

msvc.linker_flags = { "/SUBSYSTEM:WINDOWS" }
msvc.output = "test.exe"
msvc.definitions = { "MSVC" }
msvc.files = { "main.cpp" }
msvc.arch = "x64"
--- END MSVC 64 BIT TARGET

--- MSVC 32 BIT TARGET
msvc_x86 = workspace:create_msvc_target("msvc_x86") --- We create targets via the workspace so they can inherit some important values!
msvc_x86.include_paths = {
    sdl_path_msvc .. "/SDL2-2.30.2/include"
}

msvc_x86.libraries = {
    "shell32.lib",
    "SDL2.lib",
    "SDL2main.lib"
}

msvc_x86.library_paths = {
    sdl_path_msvc .. "/SDL2-2.30.2/lib/x86",
}

msvc_x86.linker_flags = { "/SUBSYSTEM:WINDOWS" }
msvc_x86.output = "test.exe"
msvc_x86.definitions = { "MSVC" }
msvc_x86.files = { "main.cpp" }
msvc_x86.arch = "x86"
--- END MSVC 32 BIT TARGET


--- MINGW 64 BIT TARGET
mingw = workspace:create_mingw_target("mingw")
mingw.arch = "x86_64"

mingw.output = "test.exe"
mingw.include_paths = { sdl_path_mingw .. "/x86_64-w64-mingw32/include" }
mingw.library_paths = { sdl_path_mingw .. "/x86_64-w64-mingw32/lib" }
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

mingw.compiler_flags = {"--verbose", "-mwindows", "-static"}
mingw.linker_flags = { "--high-entropy-va", "-subsystem", "windows", "--nxcompat", "-Bstatic"}
mingw.assets = { [sdl_path_mingw .. "/x86_64-w64-mingw32/bin/SDL2.dll"] = "SDL2.dll" }
mingw.files = { "main.cpp "}
--- END MINGW 64 BIT TARGET

--- MINGW 32 BIT TARGET
mingw_i686 = workspace:create_mingw_target("mingw_i686")
mingw_i686.arch = "i686"

mingw_i686.output = "test.exe"
mingw_i686.include_paths = { sdl_path_mingw .. "/i686-w64-mingw32/include" }
mingw_i686.library_paths = { sdl_path_mingw .. "/i686-w64-mingw32/lib" }
mingw_i686.libraries = {
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

mingw_i686.compiler_flags = {"--verbose", "-mwindows", "-static" }
mingw_i686.linker_flags = { "-subsystem", "windows", "--nxcompat", "-Bstatic"}
mingw_i686.assets = { [sdl_path_mingw .. "/i686-w64-mingw32/bin/SDL2.dll"] = "SDL2.dll" }
mingw_i686.files = { "main.cpp "}
--- END MINGW 32 BIT TARGET

--- GCC TARGET (assumes you're on Linux)
--- Make sure to install SDL2 via your package manager!
gcc = workspace:create_target("gcc")
gcc.libraries = { "SDL2" }

if workspace.arguments["linux"] ~= nil then
    gcc.definitions = { "linux" }
end

gcc.compiler_flags = {"--verbose"}
gcc.linker_flags = gcc.compiler_flags
gcc.compiler = "g++"
gcc.linker = gcc.compiler
gcc.output = "test"
gcc.files = { "main.cpp" }
--- END GCC TARGET

--- Targets should be registered after they are created and set up, not before or during!
workspace:register_target(mingw)
workspace:register_target(mingw_i686)
workspace:register_target(msvc)
workspace:register_target(msvc_x86)
workspace:register_target(gcc)

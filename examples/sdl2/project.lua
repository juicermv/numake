-- Download SDL2 for MSVC and mingw
sdl_path_mingw = workspace:download_zip("https://github.com/libsdl-org/SDL/releases/download/release-2.30.2/SDL2-devel-2.30.2-mingw.zip") .. "/SDL2-2.30.2"
sdl_path_msvc = workspace:download_zip("https:/github.com/libsdl-org/SDL/releases/download/release-2.30.2/SDL2-devel-2.30.2-VC.zip")

-- Sample. Make sure this works on your machine. Otherwise change the variables accordingly.
MSVC_LOCATION = "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC/14.39.33519"
WIN10_SDK_LOCATION = "C:/Program Files (x86)/Windows Kits/10/"
WIN10_SDK_BUILD = "10.0.22621.0"
VC_AUX = "C:/Program Files/Microsoft Visual Studio/2022/Community/VC/Auxiliary"
NET_48 = "C:/Program Files (x86)/Windows Kits/NETFXSDK/4.8"

-- MSVC 64 BIT TARGET
msvc = workspace:create_target("msvc") -- We create targets via the workspace so they can inherit some important values!
msvc.include_paths = {
    MSVC_LOCATION .. "/include",
    MSVC_LOCATION .. "/atlmfc/include",
    VC_AUX .. "/VS/include",
    WIN10_SDK_LOCATION .."/include/" .. WIN10_SDK_BUILD .. "/ucrt",
    WIN10_SDK_LOCATION .."/include/" .. WIN10_SDK_BUILD .. "/um",
    WIN10_SDK_LOCATION .."/include/" .. WIN10_SDK_BUILD .. "/shared",
    WIN10_SDK_LOCATION .."/include/" .. WIN10_SDK_BUILD .. "/winrt",
    WIN10_SDK_LOCATION .."/include/" .. WIN10_SDK_BUILD .. "/cppwinrt",
    NET_48 .. "/Include/um",
    sdl_path_msvc .. "/SDL2-2.30.2/include"
}

msvc.libraries = {
    "kernel32.lib",
    "user32.lib",
    "gdi32.lib",
    "winspool.lib",
    "comdlg32.lib",
    "advapi32.lib",
    "shell32.lib",
    "ole32.lib",
    "oleaut32.lib",
    "uuid.lib",
    "odbc32.lib",
    "odbccp32.lib",
}

msvc.library_paths = {
    MSVC_LOCATION .."/lib/x64",
    MSVC_LOCATION .."/atlmfc/lib/x64",
    VC_AUX .. "/VS/lib/x64",
    WIN10_SDK_LOCATION .. "/lib/" .. WIN10_SDK_BUILD .. "/ucrt/x64",
    WIN10_SDK_LOCATION .. "/lib/" .. WIN10_SDK_BUILD .. "/um/x64",
    NET_48 .. "/lib/um/x64",
    sdl_path_msvc .. "/SDL2-2.30.2/lib/x64",
}

msvc.linker_flags = { "/SUBSYSTEM:WINDOWS" }
msvc.use_msvc = true
msvc.compiler  = MSVC_LOCATION.."/bin/Hostx64/x64/cl.exe"
msvc.linker = msvc.compiler
msvc.output = "test.exe"
msvc.files = { "main.cpp" }
-- END MSVC 64 BIT TARGET

-- MSVC 32 BIT TARGET
msvc_x86 = workspace:create_target("msvc_x86") -- We create targets via the workspace so they can inherit some important values!
msvc_x86.include_paths = {
    MSVC_LOCATION .. "/include",
    MSVC_LOCATION .. "/atlmfc/include",
    VC_AUX .. "/VS/include",
    WIN10_SDK_LOCATION .."/include/" .. WIN10_SDK_BUILD .. "/ucrt",
    WIN10_SDK_LOCATION .."/include/" .. WIN10_SDK_BUILD .. "/um",
    WIN10_SDK_LOCATION .."/include/" .. WIN10_SDK_BUILD .. "/shared",
    WIN10_SDK_LOCATION .."/include/" .. WIN10_SDK_BUILD .. "/winrt",
    WIN10_SDK_LOCATION .."/include/" .. WIN10_SDK_BUILD .. "/cppwinrt",
    NET_48 .. "/Include/um",
    sdl_path_msvc .. "/SDL2-2.30.2/include"
}

msvc_x86.libraries = {
    "kernel32.lib",
    "user32.lib",
    "gdi32.lib",
    "winspool.lib",
    "comdlg32.lib",
    "advapi32.lib",
    "shell32.lib",
    "ole32.lib",
    "oleaut32.lib",
    "uuid.lib",
    "odbc32.lib",
    "odbccp32.lib",
}

msvc_x86.library_paths = {
    MSVC_LOCATION .."/lib/x86",
    MSVC_LOCATION .."/atlmfc/lib/x86",
    VC_AUX .. "/VS/lib/x64",
    WIN10_SDK_LOCATION .. "/lib/" .. WIN10_SDK_BUILD .. "/ucrt/x86",
    WIN10_SDK_LOCATION .. "/lib/" .. WIN10_SDK_BUILD .. "/um/x86",
    NET_48 .. "/lib/um/x86",
    sdl_path_msvc .. "/SDL2-2.30.2/lib/x86",
}

msvc_x86.linker_flags = { "/SUBSYSTEM:WINDOWS" }
msvc_x86.use_msvc = true
msvc_x86.compiler  = MSVC_LOCATION.."/bin/Hostx86/x86/cl.exe"
msvc_x86.linker = msvc_x86.compiler
msvc_x86.output = "test.exe"
msvc_x86.files = { "main.cpp" }
-- END MSVC 32 BIT TARGET


-- MINGW 64 BIT TARGET
mingw = workspace:create_target("mingw")
mingw.compiler = "x86_64-w64-mingw32-g++"
mingw.linker = mingw.compiler

mingw.output = "test.exe"
mingw.include_paths = { sdl_path_mingw .. "/x86_64-w64-mingw32/include" }
mingw.library_paths = { sdl_path_mingw .. "/x86_64-w64-mingw32/lib" }
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

mingw.compiler_flags = {"--verbose", "-mwindows", "-static", "-Wl,-Bstatic"}
mingw.linker_flags = mingw.compiler_flags
mingw.assets = { [sdl_path_mingw .. "/x86_64-w64-mingw32/bin/SDL2.dll"] = "SDL2.dll" }
mingw.files = { "main.cpp "}
mingw.use_msvc = false
-- END MINGW 64 BIT TARGET

-- MINGW 32 BIT TARGET
mingw_i686 = workspace:create_target("mingw_i686")
mingw_i686.compiler = "i686-w64-mingw32-g++"
mingw_i686.linker = mingw_i686.compiler

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

mingw_i686.compiler_flags = {"--verbose", "-mwindows", "-static", "-Wl,-Bstatic"}
mingw_i686.linker_flags = mingw_i686.compiler_flags
mingw_i686.assets = { [sdl_path_mingw .. "/i686-w64-mingw32/bin/SDL2.dll"] = "SDL2.dll" }
mingw_i686.files = { "main.cpp "}
mingw_i686.use_msvc=false
-- END MINGW 32 BIT TARGET

-- GCC TARGET (assumes you're on Linux)
-- Make sure to install SDL2 via your package manager!
gcc = workspace:create_target("gcc")
gcc.libraries = { "SDL2" }

if workspace.arguments["linux"] ~= nil then
    gcc.definitions = { "linux" }
end

gcc.use_msvc=false
gcc.compiler_flags = {"--verbose"}
gcc.linker_flags = gcc.compiler_flags
gcc.compiler = "g++"
gcc.linker = gcc.compiler
gcc.output = "test"
gcc.files = { "main.cpp" }
-- END GCC TARGET

-- Targets should be registered after they are created and set up, not before or during!
workspace:register_target(mingw)
workspace:register_target(mingw_i686)
workspace:register_target(msvc)
workspace:register_target(msvc_x86)
workspace:register_target(gcc)

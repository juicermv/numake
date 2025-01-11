--- Download SDL2 for MSVC and mingw
sdl_path_mingw = network:zip("https://github.com/libsdl-org/SDL/releases/download/release-2.30.2/SDL2-devel-2.30.2-mingw.zip") .. "/SDL2-2.30.2"
sdl_path_msvc = network:zip("https://github.com/libsdl-org/SDL/releases/download/release-2.30.2/SDL2-devel-2.30.2-VC.zip")

sdl2_project = Project (
    "SDL2 Project", -- Name
    "C++", -- Language
    "sdl2_test.exe",
    { -- Source files
        "main.cpp"
    },
    nil,
    { -- Includes
        sdl_path_msvc .. "/SDL2-2.30.2/include"
    },
    { -- Library paths
        sdl_path_msvc .. "/SDL2-2.30.2/lib/x64",
    },
    { -- Libraries
        "shell32.lib",
        "SDL2.lib",
        "SDL2main.lib"
    },
    nil,
    nil,
    { "/SUBSYSTEM:WINDOWS" }, -- Linker flags
    nil,
    nil,
    "x64", -- Architecture
    "executable" -- Project Type
)

tasks:create("x64 MSVC",
    function()
        sdl2_project.defines = { "MSVC" }
        msvc:build(sdl2_project)
    end
)

tasks:create("x86 MSVC",
    function()
        sdl2_project.defines = { "MSVC" }
        sdl2_project.arch = "x86"
        sdl2_project.lib_paths = {
            sdl_path_msvc .. "/SDL2-2.30.2/lib/x86",
        }

        msvc:build(sdl2_project)
    end
)


tasks:create("x86_64-MinGW",
    function()
        sdl2_project.arch = "x86_64"
        sdl2_project.include_paths = { sdl_path_mingw .. "/x86_64-w64-mingw32/include" }
        sdl2_project.lib_paths = { sdl_path_mingw .. "/x86_64-w64-mingw32/lib" }
        sdl2_project.libs = {
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

        sdl2_project.compiler_flags = {"--verbose", "-mwindows", "-static"}
        sdl2_project.linker_flags = { "--high-entropy-va", "-subsystem", "windows", "--nxcompat", "-Bstatic"}
        sdl2_project.asset_files = { [sdl_path_mingw .. "/x86_64-w64-mingw32/bin/SDL2.dll"] = "SDL2.dll" }

        mingw:build(sdl2_project)
    end
)

tasks:create("i686-MinGW",
    function()
        sdl2_project.arch = "i686"
        sdl2_project.include_paths = { sdl_path_mingw .. "/i686-w64-mingw32/include" }
        sdl2_project.lib_paths = { sdl_path_mingw .. "/i686-w64-mingw32/lib" }
        sdl2_project.libs = {
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

        sdl2_project.compiler_flags = {"--verbose", "-mwindows", "-static" }
        sdl2_project.linker_flags = { "-subsystem", "windows", "--nxcompat", "-Bstatic"}
        sdl2_project.asset_files = { [sdl_path_mingw .. "/i686-w64-mingw32/bin/SDL2.dll"] = "SDL2.dll" }

        mingw:build(sdl2_project)
    end
)

sdl2_gcc_project = Project(
    "SDL2 Project GCC",
    nil,
    "sdl2_test",
    {
        "main.cpp"
    },
    nil,
    nil,
    {
        "SDL2"
    },
    {
        "linux"
    },
    {
        "--verbose"
    },
    {
        "--verbose"
    },
    nil,
    nil,
    nil,
    "executable"
)

tasks:create("GCC", function()
    generic:build(sdl2_gcc_project, "g++", "g++")
end)

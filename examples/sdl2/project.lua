--- Download SDL2 for MSVC and mingw
sdl_path_mingw = network:zip("https://github.com/libsdl-org/SDL/releases/download/release-2.30.2/SDL2-devel-2.30.2-mingw.zip") .. "/SDL2-2.30.2"
sdl_path_msvc = network:zip("https://github.com/libsdl-org/SDL/releases/download/release-2.30.2/SDL2-devel-2.30.2-VC.zip")

sdl2_project = new_project("SDL2 Project", "CPP")
sdl2_project:output("sdl2_test.exe")
sdl2_project:file("main.cpp")
sdl2_project:type("Executable")

tasks:create("x64 MSVC",
    function()
        sdl2_project:lib("shell32.lib")
        sdl2_project:lib("SDL2.lib")
        sdl2_project:lib("SDL2main.lib")
        sdl2_project:arch("x64")
        sdl2_project:flag("Linker", "/SUBSYSTEM:WINDOWS")
        sdl2_project:include(sdl_path_msvc .. "/SDL2-2.30.2/include")
        sdl2_project:lib_path(sdl_path_msvc .. "/SDL2-2.30.2/lib/x64")
        sdl2_project:define("MSVC")
        msvc:build(sdl2_project)
    end
)

tasks:create("x86 MSVC",
    function()
        sdl2_project:lib("shell32.lib")
        sdl2_project:lib("SDL2.lib")
        sdl2_project:lib("SDL2main.lib")
        sdl2_project:flag("Linker", "/SUBSYSTEM:WINDOWS")
        sdl2_project:include(sdl_path_msvc .. "/SDL2-2.30.2/include")
        sdl2_project:define("MSVC")
        sdl2_project:arch("x86")
        sdl2_project:lib_path(sdl_path_msvc .. "/SDL2-2.30.2/lib/x86")

        msvc:build(sdl2_project)
    end
)


tasks:create("x86_64-MinGW",
    function()
        sdl2_project:arch("x86_64")
        sdl2_project:include(sdl_path_mingw .. "/x86_64-w64-mingw32/include")
        sdl2_project:lib_path(sdl_path_mingw .. "/x86_64-w64-mingw32/lib")

        sdl2_project:lib({
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
        })

        sdl2_project:flag("Compiler", {"--verbose", "-mwindows", "-static"})
        sdl2_project:flag("Linker", { "--high-entropy-va", "-subsystem", "windows", "--nxcompat", "-Bstatic"})
        sdl2_project:asset(sdl_path_mingw .. "/x86_64-w64-mingw32/bin/SDL2.dll", "SDL2.dll")

        mingw:build(sdl2_project)
    end
)

tasks:create("i686-MinGW",
    function()
        sdl2_project:arch("i686")
        sdl2_project:include(sdl_path_mingw .. "/i686-w64-mingw32/include")
        sdl2_project:lib_path(sdl_path_mingw .. "/i686-w64-mingw32/lib")
        sdl2_project:lib({
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
        })

        sdl2_project:flag("Compiler", {"--verbose", "-mwindows", "-static" })
        sdl2_project:flag("Linker", { "-subsystem", "windows", "--nxcompat", "-Bstatic"})
        sdl2_project:asset(sdl_path_mingw .. "/i686-w64-mingw32/bin/SDL2.dll", "SDL2.dll")

        mingw:build(sdl2_project)
    end
)


tasks:create("GCC", function()
    sdl2_gcc_project = new_project("SDL2 Project GCC", "sdl2_test")
    sdl2_gcc_project:file("main.cpp")
    sdl2_gcc_project:lib("SDL2")
    sdl2_gcc_project:define("linux")
    sdl2_gcc_project:flag("Linker", "--verbose")
    sdl2_gcc_project:flag("Compiler", "--verbose")
    sdl2_gcc_project:type("Executable")

    generic:build(sdl2_gcc_project, "g++", "g++")
end)

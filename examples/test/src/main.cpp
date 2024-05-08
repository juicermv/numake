// Simple test for numake. Will simply try to init SDL2 and return 1 if it fails.
#include <iostream>
#include <algorithm>

#ifndef MSVC
    #define SDL_MAIN_HANDLED
    #include <SDL2/SDL.h>
#else
    #include <SDL.h>
#endif

int main(int argc, char* argv[])
{
    // Initialize SDL events because other things may or may not be available on hardware
    if(SDL_Init(SDL_INIT_EVENTS) < 0)
    {
        std::cout << "SDL could not be initialized!" << std::endl
                  << "SDL_Error: " << SDL_GetError() << std::endl;
        return 1;
    }

    // Quit SDL
    SDL_Quit();
    return 0;
}
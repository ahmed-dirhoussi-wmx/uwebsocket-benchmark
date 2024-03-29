cmake_minimum_required(VERSION 3.25.0)
set(UWS_CPP_VERSION "0.0.1")
project(uws-server VERSION ${UWS_CPP_VERSION} LANGUAGES CXX C)

# ---------------------------------------------------------------------------------------
# Set UWS_MASTER_PROJECT to ON if we are building uws-server
# ---------------------------------------------------------------------------------------
# Check if owl is being used directly or via add_subdirectory, but allow overriding
if(NOT DEFINED UWS_MASTER_PROJECT)
    if(CMAKE_CURRENT_SOURCE_DIR STREQUAL CMAKE_SOURCE_DIR)
        set(UWS_MASTER_PROJECT ON)
    else()
        set(UWS_MASTER_PROJECT OFF)
    endif()
endif()

message(STATUS "Build uws-server: ${UWS_CPP_VERSION}")

# ---------------------------------------------------------------------------------------
# Set default build to release
# ---------------------------------------------------------------------------------------
if(NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE "Release" CACHE STRING "Choose Release or Debug" FORCE)
endif()

# ---------------------------------------------------------------------------------------
# Compiler config
# ---------------------------------------------------------------------------------------
if(NOT CMAKE_CXX_STANDARD)
    set(CMAKE_CXX_STANDARD 20)
    set(CMAKE_CXX_STANDARD_REQUIRED ON)
endif()

if(NOT CMAKE_C_STANDARD)
    set(CMAKE_C_STANDARD 11)
    set(CMAKE_C_STANDARD_REQUIRED ON)
endif()

# ---------------------------------------------------------------------------------------
# Fetch dependencies
# ---------------------------------------------------------------------------------------
include(FetchContent)
FetchContent_Declare(nlohmann
  GIT_REPOSITORY    https://github.com/nlohmann/json.git
  GIT_TAG           v3.11.3
)
set(JSON_BuildTests OFF CACHE INTERNAL "")
set(JSON_Install OFF CACHE INTERNAL "")

FetchContent_Declare(uwebsockets
  GIT_REPOSITORY    https://github.com/uNetworking/uWebSockets.git
  GIT_TAG           v20.62.0
)

FetchContent_MakeAvailable(nlohmann uwebsockets)

file(GLOB_RECURSE SOURCES ${uwebsockets_SOURCE_DIR}/uSockets/src/*.c)
add_library(uSockets ${SOURCES})
target_include_directories(uSockets PUBLIC ${uwebsockets_SOURCE_DIR}/uSockets/src)
target_compile_definitions(uSockets PRIVATE LIBUS_NO_SSL)

find_package(ZLIB REQUIRED)
add_library(uWebSockets INTERFACE)
target_include_directories(uWebSockets INTERFACE ${uwebsockets_SOURCE_DIR}/src/)
target_link_libraries(uWebSockets INTERFACE uSockets ${ZLIB_LIBRARIES})

# ---------------------------------------------------------------------------------------
# Define library
# ---------------------------------------------------------------------------------------
set(UWS_SRCS
  src/main.cc
  )
add_executable(uws-server ${UWS_SRCS})
target_link_libraries(uws-server PRIVATE $<BUILD_INTERFACE:nlohmann_json::nlohmann_json> uWebSockets)
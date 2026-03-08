set(CMAKE_SYSTEM_NAME Linux)
set(CMAKE_SYSTEM_PROCESSOR aarch64)

# boring-sys2 relies on a CMake toolchain file for Linux arm64 cross builds.
# Pin the actual GNU cross tools here so CMake does not fall back to the host
# compiler and produce x86_64 BoringSSL objects.
set(CMAKE_C_COMPILER aarch64-linux-gnu-gcc)
set(CMAKE_CXX_COMPILER aarch64-linux-gnu-g++)
set(CMAKE_ASM_COMPILER aarch64-linux-gnu-gcc)
set(CMAKE_AR aarch64-linux-gnu-ar)
set(CMAKE_RANLIB aarch64-linux-gnu-ranlib)
set(CMAKE_STRIP aarch64-linux-gnu-strip)

set(CMAKE_C_COMPILER_TARGET aarch64-unknown-linux-gnu)
set(CMAKE_CXX_COMPILER_TARGET aarch64-unknown-linux-gnu)
set(CMAKE_ASM_COMPILER_TARGET aarch64-unknown-linux-gnu)

set(CMAKE_FIND_ROOT_PATH /usr/aarch64-linux-gnu)
set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)

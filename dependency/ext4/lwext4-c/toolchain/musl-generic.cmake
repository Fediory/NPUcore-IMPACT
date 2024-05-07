if(NOT DEFINED ENV{ARCH})
    set(ARCH "x86_64")
else()
    set(ARCH $ENV{ARCH})
endif()

message(STATUS "Using ARCH from environment: $ENV{ARCH}")
# Name of the target
set(CMAKE_SYSTEM_NAME "Linux")
set(CMAKE_SYSTEM_PROCESSOR ${ARCH})

# Toolchain settings
set(TOOLCHAIN_PREFIX ${ARCH}-linux-musl)

set(CMAKE_C_COMPILER    ${TOOLCHAIN_PREFIX}-cc)
set(CMAKE_CXX_COMPILER  ${TOOLCHAIN_PREFIX}-c++)
set(AS                  ${TOOLCHAIN_PREFIX}-as)
set(AR                  ${TOOLCHAIN_PREFIX}-ar)
set(OBJCOPY             ${TOOLCHAIN_PREFIX}-objcopy)
set(OBJDUMP             ${TOOLCHAIN_PREFIX}-objdump)
set(SIZE                ${TOOLCHAIN_PREFIX}-size)

set(CMAKE_C_FLAGS "-Wall -fno-builtin -ffreestanding -fdata-sections -ffunction-sections" CACHE INTERNAL "c compiler flags")
set(CMAKE_CXX_FLAGS "-Wall -fno-builtin ffreestanding -fdata-sections -ffunction-sections" CACHE INTERNAL "cxx compiler flags")
set(CMAKE_ASM_FLAGS "" CACHE INTERNAL "asm compiler flags")

if(APPLE)
    set(CMAKE_EXE_LINKER_FLAGS "-dead_strip" CACHE INTERNAL "exe link flags")
else(APPLE)
    set(CMAKE_EXE_LINKER_FLAGS "-Wl,--gc-sections" CACHE INTERNAL "exe link flags")
endif(APPLE)

set(LD_FLAGS "-nolibc -nostdlib -static -no-pie --gc-sections -nostartfiles")

SET(CMAKE_C_FLAGS_DEBUG "-O0 -g -ggdb3" CACHE INTERNAL "c debug compiler flags")
SET(CMAKE_CXX_FLAGS_DEBUG "-O0 -g -ggdb3" CACHE INTERNAL "cxx debug compiler flags")
SET(CMAKE_ASM_FLAGS_DEBUG "-g -ggdb3" CACHE INTERNAL "asm debug compiler flags")

SET(CMAKE_C_FLAGS_RELEASE "-O2 -g -ggdb3" CACHE INTERNAL "c release compiler flags")
SET(CMAKE_CXX_FLAGS_RELEASE "-O2 -g -ggdb3" CACHE INTERNAL "cxx release compiler flags")
SET(CMAKE_ASM_FLAGS_RELEASE "" CACHE INTERNAL "asm release compiler flags")
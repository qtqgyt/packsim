cmake_minimum_required(VERSION 3.18)
project(packsim LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Build configuration options
option(PACKSIM_NATIVE_ARCH "Optimize for the host CPU architecture (-march=native)" ON)
option(ENABLE_CUDA "Build GPU shared library with CUDA" ON)
option(BUILD_BENCHMARKS "Build CPU standalone benchmark" ON)

# Find OpenMP for parallel CPU simulation
find_package(OpenMP REQUIRED)

# ==============================================================================
# CPU Shared Library: packopen_cpu
# ==============================================================================
set(CPU_SOURCES
    src/cpu/entry.cpp
)

add_library(packopen_cpu SHARED ${CPU_SOURCES})
target_include_directories(packopen_cpu PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
target_link_libraries(packopen_cpu PRIVATE OpenMP::OpenMP_CXX)

set_target_properties(packopen_cpu PROPERTIES
    PREFIX "lib"
    OUTPUT_NAME "packopen_cpu"
)

if(MSVC)
    target_compile_options(packopen_cpu PRIVATE /O2 /arch:AVX2 /openmp)
else()
    target_compile_options(packopen_cpu PRIVATE -O3 -fPIC)
    if(PACKSIM_NATIVE_ARCH)
        target_compile_options(packopen_cpu PRIVATE -march=native -mtune=native)
    endif()
    if(UNIX AND NOT APPLE)
        target_link_options(packopen_cpu PRIVATE -Wl,-z,nodelete)
    endif()
endif()

# Copy shared library to project root for backward-compatible script execution
add_custom_command(TARGET packopen_cpu POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
    $<TARGET_FILE:packopen_cpu>
    ${CMAKE_CURRENT_SOURCE_DIR}/$<TARGET_FILE_NAME:packopen_cpu>
    COMMENT "Copying packopen_cpu to project root"
)

# ==============================================================================
# GPU Shared Library (Optional): packopen_gpu
# ==============================================================================
if(ENABLE_CUDA)
    include(CheckLanguage)
    check_language(CUDA)
    if(CMAKE_CUDA_COMPILER)
        enable_language(CUDA)
        set(CMAKE_CUDA_STANDARD 17)
        set(CMAKE_CUDA_STANDARD_REQUIRED ON)

        if(NOT DEFINED CMAKE_CUDA_ARCHITECTURES)
            set(CMAKE_CUDA_ARCHITECTURES 75 80 86 89 90)
        endif()

        set(GPU_SOURCES
            src/gpu/entry.cu
        )

        add_library(packopen_gpu SHARED ${GPU_SOURCES})
        target_include_directories(packopen_gpu PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
        set_target_properties(packopen_gpu PROPERTIES
            PREFIX "lib"
            OUTPUT_NAME "packopen_gpu"
            CUDA_SEPARABLE_COMPILATION OFF
        )

        target_compile_options(packopen_gpu PRIVATE
            $<$<COMPILE_LANGUAGE:CUDA>:--expt-relaxed-constexpr -O3>
        )
        if(UNIX AND NOT APPLE)
            target_link_options(packopen_gpu PRIVATE -Xlinker -z,nodelete)
        endif()

        add_custom_command(TARGET packopen_gpu POST_BUILD
            COMMAND ${CMAKE_COMMAND} -E copy_if_different
            $<TARGET_FILE:packopen_gpu>
            ${CMAKE_CURRENT_SOURCE_DIR}/$<TARGET_FILE_NAME:packopen_gpu>
            COMMENT "Copying packopen_gpu to project root"
        )
    else()
        message(STATUS "CUDA compiler not found; skipping packopen_gpu")
    endif()
endif()

# ==============================================================================
# Standalone CPU Benchmark: cpu_bench
# ==============================================================================
if(BUILD_BENCHMARKS AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/bench/cpu_bench.cpp")
    add_executable(cpu_bench bench/cpu_bench.cpp)
    target_include_directories(cpu_bench PRIVATE ${CMAKE_CURRENT_SOURCE_DIR})
    target_link_libraries(cpu_bench PRIVATE OpenMP::OpenMP_CXX)
    if(PACKSIM_NATIVE_ARCH AND NOT MSVC)
        target_compile_options(cpu_bench PRIVATE -march=native -mtune=native)
    endif()
endif()

# ==============================================================================
# Rust CLI: packopener-cli
# ==============================================================================
find_program(CARGO_BIN cargo)
if(CARGO_BIN AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/cli/Cargo.toml")
    add_custom_target(packopener-cli
        COMMAND ${CARGO_BIN} build --release
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${CMAKE_CURRENT_SOURCE_DIR}/cli/target/release/packopener-cli
        ${CMAKE_CURRENT_SOURCE_DIR}/packopener-cli
        WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/cli
        COMMENT "Building Rust CLI (packopener-cli) via cargo"
    )
endif()

# ==============================================================================
# Code Generators
# ==============================================================================
find_package(Python3 COMPONENTS Interpreter QUIET)
if(Python3_Interpreter_FOUND)
    add_custom_target(tables
        COMMAND ${Python3_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/scripts/gen_tables_cu.py ${CMAKE_CURRENT_SOURCE_DIR}/src/common/tables_gen.hpp
        COMMENT "Generating C++ tables: src/common/tables_gen.hpp"
    )
    add_custom_target(tables-rs
        COMMAND ${Python3_EXECUTABLE} ${CMAKE_CURRENT_SOURCE_DIR}/scripts/gen_tables.py ${CMAKE_CURRENT_SOURCE_DIR}/cli/src/tables_gen.rs
        COMMENT "Generating Rust tables: cli/src/tables_gen.rs"
    )
endif()

# ==============================================================================
# Testing
# ==============================================================================
enable_testing()
if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/test.sh")
    add_test(NAME golden_integration_test
        COMMAND "${CMAKE_CURRENT_SOURCE_DIR}/test.sh"
        WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    )
endif()

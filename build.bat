@echo off

setlocal enabledelayedexpansion

REM Add cargo to PATH if not already there
if not defined CARGO_HOME (
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

REM --- Compiler Caching & Dynamic Linking Configuration ---
set CCACHE_ENABLE=1
set CC=ccache gcc
set CXX=ccache g++

REM Point Rust to the directory containing duckdb.lib/duckdb.dll
set DUCKDB_LIB_DIR=%~dp0\lib
REM --------------------------------------------------------
echo Building duckdb_load...

if "%1"=="" (
    echo Building debug version...
    cargo build
) else if "%1"=="debug" (
    echo Building debug version...
    cargo build
) else if "%1"=="release" (
    echo Building release version...
    cargo build --release
) else if "%1"=="asm" (
    echo Building with ASM output...
    if not exist asm mkdir asm
    cargo build --release --target x86_64-pc-windows-msvc
) else (
    echo Usage: build.bat [debug^|release^|asm]
    exit /b 1
)

if errorlevel 1 (
    echo Build failed!
    exit /b 1
)

echo Build complete!
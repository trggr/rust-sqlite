@echo off
REM Install script for ora_rust

setlocal enabledelayedexpansion

setlocal enabledelayedexpansion

REM Add cargo to PATH if not already there
if not defined CARGO_HOME (
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

REM Default deploy path
set "DEPLOY_PATH=%USERPROFILE%\bin"

REM Allow override via argument
if not "%1"=="" (
    set "DEPLOY_PATH=%1"
)

echo Building release version...
cargo build --release

if errorlevel 1 (
    echo Build failed!
    exit /b 1
)

echo Creating deploy directory: %DEPLOY_PATH%
if not exist "%DEPLOY_PATH%" mkdir "%DEPLOY_PATH%"

echo Copying executable to %DEPLOY_PATH%...
copy "target\debug\ora_rust.exe" "%DEPLOY_PATH%\ora_rust.exe"

if errorlevel 1 (
    echo Copy failed!
    exit /b 1
)

echo Installation complete!
echo ora_rust installed to: %DEPLOY_PATH%\ora_rust.exe
echo.
echo To use ora_rust, ensure %DEPLOY_PATH% is in your PATH environment variable.

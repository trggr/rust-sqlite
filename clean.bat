@echo off
REM Clean script for ora_rust

echo Cleaning ora_rust build artifacts...

cargo clean

if exist asm (
    rmdir /s /q asm
    echo Removed asm directory
)

if exist *_output.* (
    del /q *_output.*
    echo Removed output files
)

echo Clean complete!

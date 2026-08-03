@echo off
REM Run script for ora_rust

setlocal enabledelayedexpansion

REM Add cargo to PATH if not already there
if not defined CARGO_HOME (
    set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

rem cargo run --release -- -c bdev -f data\odpic_test.dat
cargo run --release -- -d data.db -f data\sample.dat
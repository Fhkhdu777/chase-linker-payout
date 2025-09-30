@echo off
setlocal enabledelayedexpansion
cd /d %~dp0

if "%RUST_LOG%"=="" set "RUST_LOG=info"

cargo run --release

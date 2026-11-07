@echo off
title DivinePort - Native PS2 Static Recompiler Launcher
cd /d "%~dp0"
if exist "DivinePort.exe" (
    start "" "DivinePort.exe"
) else if exist "target\release\DivinePort.exe" (
    start "" "target\release\DivinePort.exe"
) else (
    echo Building DivinePort native release binary...
    cargo run --release
)

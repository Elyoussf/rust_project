@echo off
setlocal

rem Set the path to your Rust project's root directory
rem cd /d path\to\your\project

rem Check if any arguments are provided
if "%~1"=="" (
    echo Error: No command provided. 2>nul
    exit /b 1
)

rem Check if the first argument is a valid command
if "%~1"=="init" (
    rem Run cargo with "init" command
    start /b cargo run --bin init 2>nul 

    rem Wait for the Rust command to finish
    :wait_init
    timeout /t 1 /nobreak >nul
    tasklist | find /i "cargo.exe" 2>nul >nul
    if not errorlevel 1 goto wait_init

    rem If cargo is no longer running, set the hidden attribute for the .rgit directory and exit
    attrib +h ".rgit" 2>nul
    popd
    exit /b
) else if "%~1"=="add" (
    rem Run cargo with "add" command and passed arguments
    start /b cargo run --bin add %* 2>nul

    rem Wait for the Rust command to finish
    :wait_add
    timeout /t 1 /nobreak >nul
    tasklist | find /i "cargo.exe" >nul
    if not errorlevel 1 goto wait_add

    rem If cargo is no longer running, set the hidden attribute for the .rgit directory and exit
    attrib +h ".rgit" 1>nul 2>nul
    popd
    exit /b
) else if "%~1"=="commit" (
    rem Shift the arguments to handle the double dash properly
    shift
    shift
    start /b cargo run --bin %* 2>nul

    rem Wait for the Rust command to finish
    :wait_commit
    timeout /t 1 /nobreak >nul
    tasklist | find /i "cargo.exe" >nul
    if not errorlevel 1 goto wait_commit

    rem If cargo is no longer running, set the hidden attribute for the .rgit directory and exit
    attrib +h ".rgit" 1>nul 2>nul
    popd
    exit /b
) else (
    echo Error: Unknown command "%~1"
    popd
    exit /b 1
)

endlocal

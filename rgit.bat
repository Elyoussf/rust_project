@echo off
setlocal

rem Check if any arguments are provided
if "%1"=="" (
    echo Usage: rgit <init|add> [arguments...]
    exit /b 1
)

rem Check if the first argument is a valid command
if "%1"=="init" (
    rem Run cargo with "init" command
    start /b cargo run --bin init  

    rem Wait for the Rust command to finish
    :wait_init
    tasklist | find /i "cargo.exe" >nul
    if errorlevel 1 (
        rem If cargo is no longer running, set the hidden attribute for the .rgit directory and exit
        attrib +h ".rgit"
        exit /b
    )
    rem If cargo is still running, wait for 1 second and check again
    timeout /t 1 /nobreak >nul
    goto wait_init

) else if "%1"=="add" (
    rem Run cargo with "add" command and passed arguments
    start /b cargo run --bin add %*  2>nul

    rem Wait for the Rust command to finish
    :wait_add
    tasklist | find /i "cargo.exe" >nul
    if errorlevel 1 (
        rem If cargo is no longer running, set the hidden attribute for the .rgit directory and exit
        attrib +h ".rgit"
        exit /b
    )
    rem If cargo is still running, wait for 1 second and check again
    timeout /t 1 /nobreak >nul
    goto wait_add

) else (
    echo Usage: rgit <init|add> [arguments...]
    exit /b 1
)

endlocal

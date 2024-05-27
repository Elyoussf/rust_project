@echo off
setlocal


if "%1"=="init" (

    
    start /b cargo run --bin init

    :wait
    tasklist | find /i "cargo.exe" >nul
    if errorlevel 1 (
   
        goto continue
    )
    

    timeout /t 1 /nobreak >nul
    goto wait
)

:continue


attrib +h ".rgit"



endlocal

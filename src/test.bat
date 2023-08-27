@echo off
setlocal enabledelayedexpansion

set FILEPATH=%APPDATA%\Workspacer\file.txt

Workspacer.exe


if not exist "%FILEPATH%" (
    echo File not found: %FILEPATH%
    exit /b
)

for /f "tokens=*" %%a in (%FILEPATH%) do (
    set FIRST_LINE=%%a
    goto done
)

:done
cd !FIRST_LINE!

endlocal

@echo off
setlocal enabledelayedexpansion

echo Before running Workspacer.exe
type "%APPDATA%\Workspacer\file.txt"

Workspacer.exe %1 %2 %3 %4

type "%APPDATA%\Workspacer\file.txt"

set FILEPATH=%APPDATA%\Workspacer\file.txt
if not exist "%FILEPATH%" (
    echo File not found: %FILEPATH%
    exit /b
)

for /f "tokens=*" %%a in ('type "%FILEPATH%"') do (
    cmd /k cd %%a
    goto done
)

:done

endlocal

# Turn off the echo of commands
$DebugPreference = "SilentlyContinue"

# Equivalent to 'setlocal enabledelayedexpansion'
Set-StrictMode -Version Latest

# Equivalent to 'echo Before running Workspacer.exe'
Write-Host "Before running Workspacer.exe"

# Equivalent to 'type "%APPDATA%\Workspacer\file.txt"'
Get-Content "$env:APPDATA\Workspacer\file.txt"

# Pass all arguments to Workspacer.exe
& "Workspacer.exe" @args

# Equivalent to 'type "%APPDATA%\Workspacer\file.txt"'
Get-Content "$env:APPDATA\Workspacer\file.txt"

# Initialize the variable FILEPATH
$FILEPATH = "$env:APPDATA\Workspacer\file.txt"

# Check if the file exists
if (-Not (Test-Path $FILEPATH)) {
    Write-Host "File not found: $FILEPATH"
    exit
}

# Loop through each line in the file
Get-Content $FILEPATH | ForEach-Object {
    Set-Location $_
    break
}

# Wait for user input before closing the window
Read-Host "Press Enter to exit"


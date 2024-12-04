@echo off

set "script_dir=%~dp0"
set "extractor_dir=%script_dir%extractor\"

if exist "%extractor_dir%" (
    call "%extractor_dir%run_gradle.bat"
    if %errorlevel% neq 0 exit /b %errorlevel%

    call "%script_dir%move_assets.bat"
    if %errorlevel% neq 0 exit /b %errorlevel%

    @RD /S /Q "%extractor_dir%run"
) else (
    echo "make sure to run this script as-is in the Pumpkin source root!"
    exit /b 1
)


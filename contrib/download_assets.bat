@echo off

set "script_dir=%~dp0"
set "root_dir=%script_dir%..\"
set "extractor_dir=%root_dir%extractor\"
set "output_dir=%extractor_dir%run\pumpkin_extractor_output\"

if exist "%extractor_dir%" (
    md %output_dir% 2> nul
    bitsadmin /transfer "download_pumpkin_assets" /download /priority FOREGROUND "https://pumpkin.kralverde.dev/assets/artifacts/pumpkin-assets.zip" "%output_dir%pumpkin-assets.zip"
    if %errorlevel% neq 0 exit /b %errorlevel%

    powershell -command "Expand-Archive -Force '%output_dir%pumpkin-assets.zip' '%output_dir%'"
    if %errorlevel% neq 0 exit /b %errorlevel%

    call "%script_dir%move_assets.bat"
    if %errorlevel% neq 0 exit /b %errorlevel%

    @RD /S /Q "%extractor_dir%run"
    if %errorlevel% neq 0 exit /b %errorlevel%
) else (
    echo "make sure to run this script as-is in the Pumpkin source root!"
    exit /b 1
)


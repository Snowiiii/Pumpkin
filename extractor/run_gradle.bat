@echo off

set "script_dir=%~dp0"
set "run_dir=%script_dir%run\"

if not exist "%run_dir%" mkdir "%run_dir%"
set "file_path=%run_dir%eula.txt"

echo eula=true> "%file_path%"

pushd "%script_dir%"
gradlew.bat "runServer"
popd

if %ERRORLEVEL% GEQ 1 exit /B 1


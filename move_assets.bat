@echo off

set "script_dir=%~dp0"
set "extractor_dir=%script_dir%extractor\"
set "file_map=%extractor_dir%file_map.dat"

for /F "tokens=*" %%A in (%file_map%) do call :Move %%A
goto End

:Move
set "first=%1"
set "second=%2"
if /i "%first:~0,1%"=="#" goto :eof
for /F "delims=" %%i in ("%script_dir%%second%") do set "dest_dir=%%~dpi"

::echo "Creating %dest_dir%"
md %dest_dir% 2> nul

set "first=%first:/=\\%"
set "second=%second:/=\\%"
::echo "Moving %extractor_dir%%first% -> %script_dir%%second%"
move "%extractor_dir%%first%" "%script_dir%%second%"
if %errorlevel% neq 0 exit /b %errorlevel%

goto :eof

:End


@echo off

set "root_dir=%~dp0\..\"
set "extractor_dir=%root_dir%extractor\"
set "output_dir=%extractor_dir%run\pumpkin_extractor_output\"
set "file_map=%extractor_dir%file_map.dat"
set "shafile=%extractor_dir%expected_sha256.dump"

for /F "tokens=*" %%A in (%shafile%) do call :Check_Hash %%A
if %errorlevel% neq 0 exit /b %errorlevel%

for /F "tokens=*" %%A in (%file_map%) do call :Move %%A
if %errorlevel% neq 0 exit /b %errorlevel%
goto End

:Move
set "first=%1"
set "second=%2"
if /i "%first:~0,1%"=="#" goto :eof
for /F "delims=" %%i in ("%root_dir%%second%") do set "dest_dir=%%~dpi"

md %dest_dir% 2> nul

set "first=%first:/=\\%"
set "second=%second:/=\\%"
move "%output_dir%%first%" "%root_dir%%second%"
if %errorlevel% neq 0 goto :Fail %errorlevel%
goto :eof

:Check_Hash
set "expected_hash=%1"
set "file=%2"
if not exist "%output_dir%%file%" (
    echo "file %output_dir%%file% does not exist!"
    goto :Fail 1
)

for /f "delims=" %%i in ('certutil -hashfile "%output_dir%%file%" SHA256 ^| find /i /v "sha256" ^| find /i /v "certutil"') do set "current_hash=%%i"
if %errorlevel% neq 0 goto :Fail %errorlevel%

set "current_hash=%current_hash: =%"

if "%current_hash%"=="" (
    echo "failed to get hash for %file%"
    goto :Fail 1
)

if not "%current_hash%"=="%expected_hash%" (
    echo "warning: hash mismatch! %current_hash% vs %upper_hash%"
    goto :Fail 1
)
goto :eof

:End
echo "Successfully moved asset files!"
goto :eof

:Fail
exit %1

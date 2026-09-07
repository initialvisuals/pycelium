@echo off
setlocal EnableExtensions
cd /d "%~dp0"

REM Build first (no success-pause - we go straight to the window).
REM build.bat still writes build.log and always pauses when double-clicked.
call "%~dp0build.bat" /nopause
if errorlevel 1 (
  echo.
  echo Build failed. Open build.log in the repo root if this window is gone.
  echo.
  pause
  exit /b 1
)

set "BIN=%~dp0target\release\pycelium-win.exe"
if not exist "%BIN%" (
  echo Expected binary missing: %BIN%
  echo Open build.log in the repo root if this window is gone.
  pause
  exit /b 1
)

REM Quality presets: demo / performant / beast. Other flags pass through
REM (e.g. --preset demo --drift, --bench 60).
set "ARGS=%*"
if /I "%~1"=="demo" goto :preset
if /I "%~1"=="performant" goto :preset
if /I "%~1"=="beast" goto :preset
goto :launch

:preset
set "ARGS=--preset %~1"
if not "%~2"=="" set "ARGS=%ARGS% %~2 %~3 %~4 %~5 %~6 %~7 %~8 %~9"

:launch
echo.
echo Launching Pycelium GPU mesocosm
echo   %BIN% %ARGS%
echo Close the window to return here.
echo.

REM Run in this console so we wait on the real process.
REM Do not `start`+detach, and do not `timeout /t` (feel-lab StartServer.bat:
REM Git Bash / Firefox timeout.exe spam "Invalid time interval" and can
REM drop the child early). Extra args pass through (presets / --drift / --bench).
"%BIN%" %ARGS%
set "RC=%ERRORLEVEL%"

echo.
echo pycelium-win exited with code %RC%.
pause
exit /b %RC%

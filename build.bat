@echo off
setlocal EnableExtensions
cd /d "%~dp0"

REM Always pause on double-click. /nopause is only for run.bat
REM so it can launch the GPU window without a mid-script keypress.
set "NOPAUSE=0"
if /I "%~1"=="/nopause" set "NOPAUSE=1"

set "LOG=%~dp0build.log"
set "BIN=%~dp0target\release\pycelium-win.exe"
set "RC=0"
set "STARTED=%DATE% %TIME%"

REM Double-click PATH is often empty of rustup. Prefer the user cargo bin.
if exist "%USERPROFILE%\.cargo\bin" (
  set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
)

echo.
echo Started: %STARTED%
echo Building Pycelium GPU mesocosm - release, package pycelium-win
echo   cargo build --release -p pycelium-win
echo   log: %LOG%
echo.

> "%LOG%" echo Pycelium GPU mesocosm release build
>> "%LOG%" echo Started: %STARTED%
>> "%LOG%" echo cargo build --release -p pycelium-win
>> "%LOG%" echo.

where cargo >nul 2>&1
if errorlevel 1 (
  echo cargo was not found.
  echo Install Rust from https://rustup.rs then double-click build.bat again.
  echo rustup puts cargo in %%USERPROFILE%%\.cargo\bin - this script already adds that folder.
  echo.
  echo cargo was not found.>> "%LOG%"
  echo Install Rust from https://rustup.rs>> "%LOG%"
  set "RC=1"
  goto :finish
)

call :tee_cargo
set "RC=%ERRORLEVEL%"
if not "%RC%"=="0" (
  echo.
  echo cargo build --release -p pycelium-win failed.
  echo Open build.log in the repo root if this window is gone.
  echo cargo build --release -p pycelium-win failed. code %RC%>> "%LOG%"
  goto :finish
)

echo.
if exist "%BIN%" (
  echo Binary: %BIN%
  echo Binary: %BIN%>> "%LOG%"
  echo.
  echo pycelium-win.exe last write time and size:
  echo pycelium-win.exe last write time and size:>> "%LOG%"
  dir /T:W "%BIN%"
  dir /T:W "%BIN%" >> "%LOG%"
) else (
  echo Build succeeded but %BIN% is missing. Check cargo output above or build.log.
  echo Build succeeded but binary missing: %BIN%>> "%LOG%"
  set "RC=1"
  goto :finish
)

:finish
echo.
echo Finished: %DATE% %TIME%
echo Finished: %DATE% %TIME%>> "%LOG%"
if not "%RC%"=="0" (
  echo Result: FAILED  code %RC%
  echo Result: FAILED  code %RC%>> "%LOG%"
) else (
  echo Result: OK
  echo Result: OK>> "%LOG%"
)
echo.
echo If this window closed on its own, open build.log in the repo root.
echo.
if "%NOPAUSE%"=="1" exit /b %RC%
echo Press any key to close this window.
pause
exit /b %RC%

:tee_cargo
REM Live cargo on the console, same text appended to build.log.
REM *>&1 is PowerShell redirect so cmd.exe does not steal 2>&1.
REM No parenthesized blocks here: %ERRORLEVEL% would expand too early.
where powershell.exe >nul 2>&1
if not errorlevel 1 goto :tee_ps
echo powershell.exe not found; cargo output goes to build.log, then typed back.
cargo build --release -p pycelium-win >> "%LOG%" 2>&1
set "TEE_RC=%ERRORLEVEL%"
echo ---- build.log ----
type "%LOG%"
exit /b %TEE_RC%

:tee_ps
powershell.exe -NoProfile -ExecutionPolicy Bypass -Command "& cargo build --release -p pycelium-win *>&1 | Tee-Object -FilePath 'build.log' -Append; if ($null -ne $global:LASTEXITCODE) { exit [int]$global:LASTEXITCODE }; exit 0"
exit /b %ERRORLEVEL%

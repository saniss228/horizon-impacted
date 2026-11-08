@echo off
setlocal DisableDelayedExpansion
title Impact Horizon Native

rem This launcher runs a separate executable. It never replaces the original.
set "CLIENT=%~dp0horizon-client-native.exe"
set "MOD=%~dp0impact_horizon_native.hmod"
set "RES=%~dp0.local\horizon-client\26.3"
if defined HORIZON_RESOURCES set "RES=%HORIZON_RESOURCES%"
if exist "%~dp0resources-path.txt" set /p "RES=" < "%~dp0resources-path.txt"

if not defined APPDATA (
    echo ERROR: APPDATA is not defined.
    pause
    exit /b 1
)
if not exist "%CLIENT%" (
    echo ERROR: The modified client was not found: "%CLIENT%"
    echo Extract the complete ZIP into a separate folder first.
    pause
    exit /b 1
)
if not exist "%MOD%" (
    echo ERROR: The in-game interface was not found: "%MOD%"
    pause
    exit /b 1
)
if not exist "%RES%\horizon-resources.json" (
    echo ERROR: Prepared Horizon 26.3 resources were not found:
    echo "%RES%"
    echo Set HORIZON_RESOURCES or create resources-path.txt next to this script.
    echo Read README_RU.txt for details. Do not include quotes in that file.
    pause
    exit /b 1
)
set "GAME_HOME=%APPDATA%\horizon"
if not exist "%GAME_HOME%\" mkdir "%GAME_HOME%" >nul 2>&1
pushd "%GAME_HOME%" >nul 2>&1
if errorlevel 1 (
    echo ERROR: Cannot open "%GAME_HOME%".
    pause
    exit /b 1
)

rem Session-only: do not open Agent Control even if the old setting is on.
rem The modules run in this same game process; no external panel is launched.
set "IMPACT_NATIVE_DISABLE_CONTROL=1"
echo Starting the separate Impact Horizon Native client...
echo In a world press F8, or type /ih menu.
"%CLIENT%" --resources "%RES%" --dev-mod "%MOD%" %*
set "GAME_EXIT=%ERRORLEVEL%"
popd
if not "%GAME_EXIT%"=="0" (
    echo.
    echo Horizon exited with code %GAME_EXIT%.
    echo Copy the messages above or take a screenshot.
    pause
)
exit /b %GAME_EXIT%

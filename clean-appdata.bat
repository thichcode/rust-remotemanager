@echo off
:: Clean Hermes Remote Manager AppData
:: Run this BEFORE starting a new portable build to clear stale cache/state

set "APP=%APPDATA%\com.hermes.remote-manager"

if not exist "%APP%" (
    echo [CLEAN] AppData folder not found: %APP%
    echo Nothing to clean.
    pause
    exit /b 0
)

echo ================================================
echo   Hermes Remote Manager - Data Cleaner
echo ================================================
echo.
echo Target: %APP%
echo.

:: Show what's inside
echo Current contents:
echo.
dir /b "%APP%" 2>nul || echo   (folder is empty)
echo.

:: Count items
set COUNT=0
for /d %%D in ("%APP%\*") do set /a COUNT+=1
for %%F in ("%APP%\*") do set /a COUNT+=1

if %COUNT% equ 0 (
    echo Nothing to clean.
    pause
    exit /b 0
)

echo Found %COUNT% item(s).
echo.
set /p CONFIRM=Delete ALL data in %APP%? [y/N]:
if /i not "%CONFIRM%"=="y" (
    echo Cancelled.
    pause
    exit /b 0
)

echo.
echo Deleting...

:: Delete all items inside the folder (keep folder itself)
for /d %%D in ("%APP%\*") do (
    echo   RD /S /Q "%%D"
    rd /s /q "%%D" 2>nul
)
for %%F in ("%APP%\*") do (
    echo   DEL /F /Q "%%F"
    del /f /q "%%F" 2>nul
)

:: Also delete the folder itself
echo.
echo Removing folder %APP%...
rd /s /q "%APP%" 2>nul

if exist "%APP%" (
    echo.
    echo [FAIL] Some items could not be deleted.
    echo Make sure the app is NOT running and try again.
) else (
    echo.
    echo [OK] All data cleared successfully.
)

echo.
pause
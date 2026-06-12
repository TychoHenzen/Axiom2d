@echo off
setlocal

echo [gps_companion deploy] do %DATE% %TIME%
echo.

cd /d "%~dp0"

echo [1/3] Getting dependencies...
call C:\Users\siriu\flutter\bin\flutter.bat pub get
if %ERRORLEVEL% neq 0 goto :error

echo.
echo [2/3] Building release APK...
call C:\Users\siriu\flutter\bin\flutter.bat build apk --release
if %ERRORLEVEL% neq 0 goto :error

echo OK: %~dp0build\app\outputs\flutter-apk\app-release.apk

echo.
echo [3/3] Installing to device...
call C:\Users\siriu\flutter\bin\flutter.bat install --release
if %ERRORLEVEL% neq 0 goto :error

echo.
echo Done. App installed.
exit /b 0

:error
echo.
echo FAILED with exit code %ERRORLEVEL%
exit /b %ERRORLEVEL%

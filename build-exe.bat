@echo off
setlocal
echo =========================================
echo    LiquidLauncher Windows EXE Builder    
echo =========================================

where bun >nul 2>&1
if %ERRORLEVEL% equ 0 (
    echo [*] Using bun package manager...
    call bun install
    call bun run build
    call bun run tauri build --bundles nsis
) else (
    where npm >nul 2>&1
    if %ERRORLEVEL% equ 0 (
        echo [*] Using npm package manager...
        call npm install
        call npm run build
        call npx tauri build --bundles nsis
    ) else (
        echo [!] Error: Neither bun nor npm found in PATH.
        echo Please install Bun (https://bun.sh) or Node.js.
        pause
        exit /b 1
    )
)

echo.
echo =========================================
echo           Build Finished!                
echo =========================================
echo Installer location:  src-tauri\target\release\bundle\nsis\
echo Standalone location: src-tauri\target\release\liquidlauncher.exe
echo.
pause


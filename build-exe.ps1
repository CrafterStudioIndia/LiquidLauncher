Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "   LiquidLauncher Windows EXE Builder    " -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

if (Get-Command bun -ErrorAction SilentlyContinue) {
    Write-Host "[*] Using bun..." -ForegroundColor Yellow
    bun install
    bun run build
    bun run tauri build --bundles nsis
} elseif (Get-Command npm -ErrorAction SilentlyContinue) {
    Write-Host "[*] Using npm..." -ForegroundColor Yellow
    npm install
    npm run build
    npx tauri build --bundles nsis
} else {
    Write-Error "Neither bun nor npm found. Please install Bun (https://bun.sh) or Node.js."
    exit 1
}

Write-Host "`n=========================================" -ForegroundColor Green
Write-Host "          Build Finished!                " -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
Write-Host "Installer:  src-tauri\target\release\bundle\nsis\"
Write-Host "Standalone: src-tauri\target\release\liquidlauncher.exe"


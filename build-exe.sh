#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "========================================="
echo "   LiquidLauncher Windows EXE Builder    "
echo "========================================="

if ! command -v bun >/dev/null 2>&1; then
    echo "[!] Error: 'bun' is not installed or not in PATH."
    exit 1
fi

cd "$SCRIPT_DIR"

# 1. Native Windows environment check
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" || "$OSTYPE" == "cygwin" ]]; then
    echo "[*] Windows environment detected."
    echo "[*] Building Windows frontend & standalone EXE..."
    bun install
    bun run build
    bun run tauri build --no-bundle
    
    EXE_OUT="src-tauri/target/release/liquidlauncher.exe"
    echo ""
    echo "[*] Standalone executable compiled: $EXE_OUT"
    
    if command -v iscc >/dev/null 2>&1; then
        echo "[*] Running Inno Setup Compiler (ISCC)..."
        iscc packaging/windows/setup.iss
        echo "[*] Inno Setup installer generated in src-tauri/target/release/bundle/inno/"
    else
        echo "[!] 'iscc' not found in PATH. Install Inno Setup 6 to compile the installer setup.iss"
    fi
    exit 0
fi

# 2. Linux cross-compilation check
TARGET="x86_64-pc-windows-gnu"
echo "[*] Linux host detected. Checking cross-compilation toolchain for target: $TARGET..."

HAS_TARGET=0
if command -v rustup >/dev/null 2>&1; then
    if rustup target list | grep -q "$TARGET (installed)"; then
        HAS_TARGET=1
    fi
elif command -v rustc >/dev/null 2>&1; then
    SYSROOT="$(rustc --print sysroot 2>/dev/null || true)"
    if [ -d "$SYSROOT/lib/rustlib/$TARGET" ]; then
        HAS_TARGET=1
    fi
fi

HAS_MINGW=0
if command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
    HAS_MINGW=1
fi

if [ "$HAS_TARGET" -eq 1 ] && [ "$HAS_MINGW" -eq 1 ]; then
    echo "[*] Toolchain verified (rust target $TARGET and MinGW-w64 available)."
    echo "[*] Compiling standalone Windows liquidlauncher.exe (without NSIS makensis dependency)..."
    bun install
    bun run build
    bun run tauri build --target "$TARGET" --no-bundle
    
    EXE_PATH="src-tauri/target/$TARGET/release/liquidlauncher.exe"
    echo ""
    echo "========================================="
    echo "     Windows Compilation Succeeded!      "
    echo "========================================="
    echo "Generated Windows binary:"
    ls -lh "$EXE_PATH"
    echo ""
    echo "[*] Inno Setup script is prepared at: packaging/windows/setup.iss"
    
    if command -v iscc >/dev/null 2>&1; then
        echo "[*] Found iscc, building Inno Setup installer..."
        iscc packaging/windows/setup.iss
    elif [ -f "node_modules/.bin/innosetup-compiler" ]; then
        echo "[*] Running npm innosetup-compiler on packaging/windows/setup.iss..."
        ./node_modules/.bin/innosetup-compiler packaging/windows/setup.iss || true
    else
        echo "[i] To package into an installer with Inno Setup:"
        echo "    - Run: npm run iss (powered by innosetup-compiler)"
        echo "    - On Windows: Run ISCC on 'packaging/windows/setup.iss' or install Inno Setup 6"
        echo "    - Standalone executable is ready to use immediately at: $EXE_PATH"
    fi
else
    echo ""
    echo "[!] Cross-compilation prerequisites for Windows ($TARGET) are missing:"
    [ "$HAS_TARGET" -eq 0 ] && echo "    - Rust target '$TARGET' is missing (install via: rustup target add $TARGET)"
    [ "$HAS_MINGW" -eq 0 ] && echo "    - MinGW GCC is missing (on Arch: sudo pacman -S mingw-w64-gcc; on Ubuntu: sudo apt install gcc-mingw-w64)"
    exit 1
fi

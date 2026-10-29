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

# 1. Check if running directly inside a Windows shell (Git Bash, MSYS2, CYGWIN)
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" || "$OSTYPE" == "cygwin" ]]; then
    echo "[*] Windows environment detected."
    echo "[*] Building Windows frontend & NSIS setup EXE..."
    bun install
    bun run build
    bun run tauri build --bundles nsis
    echo ""
    echo "========================================="
    echo "          Build Succeeded!              "
    echo "========================================="
    echo "Setup EXE location:      src-tauri/target/release/bundle/nsis/"
    echo "Standalone EXE location: src-tauri/target/release/liquidlauncher.exe"
    exit 0
fi

# 2. Linux host cross-compilation check
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
    echo "[*] Building frontend & cross-compiling Windows NSIS bundle..."
    bun install
    bun run build
    bun run tauri build --target "$TARGET" --bundles nsis
    echo ""
    echo "========================================="
    echo "     Cross-Compilation Succeeded!        "
    echo "========================================="
    echo "Generated output in src-tauri/target/$TARGET/release/bundle/nsis/"
else
    echo ""
    echo "[!] Cross-compilation prerequisites for Windows ($TARGET) are not installed on this host:"
    [ "$HAS_TARGET" -eq 0 ] && echo "    - Rust target '$TARGET' is missing (install via: rustup target add $TARGET)"
    [ "$HAS_MINGW" -eq 0 ] && echo "    - MinGW GCC is missing (on Arch: sudo pacman -S mingw-w64-gcc; on Ubuntu: sudo apt install gcc-mingw-w64)"
    echo ""
    echo "Recommended ways to build the Windows .exe:"
    echo "  A. On Windows:"
    echo "     Run 'build-exe.bat' or 'build-exe.ps1', or run:"
    echo "       bun install && bun run build && bun run tauri build --bundles nsis"
    echo ""
    echo "  B. Via GitHub Actions:"
    echo "     The CI workflow (.github/workflows/build.yml) automatically compiles"
    echo "     the Windows installer and executable on a native 'windows-latest' runner."
    echo ""
    echo "  C. Set up cross-compilation on Linux:"
    echo "     1. Install mingw-w64: sudo pacman -S mingw-w64-gcc (Arch) or sudo apt install gcc-mingw-w64 (Ubuntu)"
    echo "     2. Add rust target: rustup target add x86_64-pc-windows-gnu"
    echo "     3. Re-run this script."
    exit 1
fi


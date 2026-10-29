#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "========================================="
echo "   LiquidLauncher AppImage Build Script  "
echo "========================================="

# 1. Check prerequisites
if ! command -v bun >/dev/null 2>&1; then
    echo "[!] Error: 'bun' is not installed or not in PATH."
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "[!] Error: 'cargo' is not installed or not in PATH."
    exit 1
fi

# 2. Check bundled LiquidBounce jar
JAR_SRC="$ROOT_DIR/LiquidBounce/build/libs/liquidbounce-0.37.0.jar"
JAR_FALLBACK="$ROOT_DIR/liquidbounce-0.37.0-ryoku.jar"

if [ -f "$JAR_SRC" ]; then
    echo "[*] Found compiled mod jar: $JAR_SRC"
    if [ ! -f "$JAR_FALLBACK" ] || [ "$JAR_SRC" -nt "$JAR_FALLBACK" ]; then
        echo "[*] Refreshing $JAR_FALLBACK from $JAR_SRC..."
        cp "$JAR_SRC" "$JAR_FALLBACK"
    fi
elif [ -f "$JAR_FALLBACK" ]; then
    echo "[*] Found release mod jar: $JAR_FALLBACK"
else
    echo "[!] Warning: No mod jar found at $JAR_SRC or $JAR_FALLBACK."
    echo "[!] LiquidBounce should be built first via './gradlew build' in LiquidBounce/."
fi

# 3. Build frontend
echo "[*] Installing frontend dependencies and compiling assets..."
cd "$SCRIPT_DIR"
bun install
bun run build

# 4. Build AppImage bundle
echo "[*] Building Tauri AppImage bundle (this compiles Rust backend in release mode)..."
bun run tauri build --bundles appimage

# 5. Output result location
APPIMAGE_DIR="$SCRIPT_DIR/src-tauri/target/release/bundle/appimage"
if [ -d "$APPIMAGE_DIR" ]; then
    echo ""
    echo "========================================="
    echo "          Build Succeeded!              "
    echo "========================================="
    echo "Generated AppImage:"
    find "$APPIMAGE_DIR" -maxdepth 1 -type f -name "*.AppImage" -exec ls -lh {} +
    echo ""
    echo "To test run the AppImage:"
    echo "  chmod +x $APPIMAGE_DIR/*.AppImage"
    echo "  $APPIMAGE_DIR/*.AppImage"
else
    echo "[*] Build completed. Check $SCRIPT_DIR/src-tauri/target/release/ for output binaries."
fi


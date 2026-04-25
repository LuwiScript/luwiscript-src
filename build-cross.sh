#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VERSION="${1:-0.1.0}"
OUTPUT_DIR="$SCRIPT_DIR/dist"

mkdir -p "$OUTPUT_DIR"

echo "=== LuwiScript Compiler v$VERSION - Cross-Platform Build ==="
echo ""

build_target() {
  local target="$1"
  local platform_name="$2"

  echo "--- Building for $platform_name ($target) ---"

  if ! rustup target list --installed | grep -q "$target"; then
    echo "Adding target $target..."
    rustup target add "$target"
  fi

  cargo build --bin luwic --workspace --release --target "$target"

  local bin_path="target/$target/release/luwic"
  if [ -f "$bin_path" ]; then
    local archive_dir="$OUTPUT_DIR/luwic-$platform_name-v$VERSION"
    mkdir -p "$archive_dir"
    cp "$bin_path" "$archive_dir/luwic"
    chmod +x "$archive_dir/luwic"
    cp "$SCRIPT_DIR/LICENSE" "$archive_dir/"
    cp "$SCRIPT_DIR/README.md" "$archive_dir/"

    cd "$OUTPUT_DIR"
    tar -czf "luwic-$platform_name-v$VERSION.tar.gz" "luwic-$platform_name-v$VERSION"
    rm -rf "$archive_dir"

    echo "  -> $OUTPUT_DIR/luwic-$platform_name-v$VERSION.tar.gz"
  else
    echo "  ERROR: Binary not found at $bin_path"
    return 1
  fi
  echo ""
}

echo "Available targets:"
echo "  1) linux-x86_64   (x86_64-unknown-linux-gnu)"
echo "  2) linux-aarch64  (aarch64-unknown-linux-gnu)"
echo "  3) macos-x86_64   (x86_64-apple-darwin)"
echo "  4) macos-aarch64  (aarch64-apple-darwin)"
echo "  5) all"
echo ""

if [ -n "${2:-}" ]; then
  SELECTION="$2"
else
  read -rp "Select target(s) [1-5]: " SELECTION
fi

case "$SELECTION" in
  1) build_target "x86_64-unknown-linux-gnu" "linux-x86_64" ;;
  2)
    echo "Note: aarch64 Linux cross-compilation requires gcc-aarch64-linux-gnu"
    build_target "aarch64-unknown-linux-gnu" "linux-aarch64"
    ;;
  3) build_target "x86_64-apple-darwin" "macos-x86_64" ;;
  4) build_target "aarch64-apple-darwin" "macos-aarch64" ;;
  5)
    build_target "x86_64-unknown-linux-gnu" "linux-x86_64"
    build_target "aarch64-unknown-linux-gnu" "linux-aarch64"
    build_target "x86_64-apple-darwin" "macos-x86_64"
    build_target "aarch64-apple-darwin" "macos-aarch64"
    ;;
  *)
    echo "Invalid selection: $SELECTION"
    exit 1
    ;;
esac

echo "=== Build complete ==="
ls -lh "$OUTPUT_DIR"/*.tar.gz 2>/dev/null || true

#!/bin/sh
# Build the Godot extension and copy it where the Godot project loads it from.
# PROFILE=release for a release build; UNIVERSAL=1 on macOS for an arm64 + x86_64 library.
set -eu
cd "$(dirname "$0")/.."

profile=${PROFILE:-debug}
flag=""
[ "$profile" = release ] && flag="--release"
bin=elite-godot/project/bin
mkdir -p "$bin"

case "$(uname -s)" in
  Darwin)
    if [ "${UNIVERSAL:-0}" = 1 ]; then
      for target in aarch64-apple-darwin x86_64-apple-darwin; do
        rustup target add "$target"
        cargo build -p elite-godot $flag --target "$target"
      done
      lipo -create \
        "target/aarch64-apple-darwin/$profile/libelite_godot.dylib" \
        "target/x86_64-apple-darwin/$profile/libelite_godot.dylib" \
        -output "$bin/libelite_godot.macos.dylib"
    else
      cargo build -p elite-godot $flag
      cp "target/$profile/libelite_godot.dylib" "$bin/libelite_godot.macos.dylib"
    fi
    ;;
  Linux)
    cargo build -p elite-godot $flag
    cp "target/$profile/libelite_godot.so" "$bin/libelite_godot.linux.x86_64.so"
    ;;
  MINGW* | MSYS* | CYGWIN*)
    cargo build -p elite-godot $flag
    cp "target/$profile/elite_godot.dll" "$bin/elite_godot.windows.x86_64.dll"
    ;;
  *)
    echo "Unsupported platform: $(uname -s)" >&2
    exit 1
    ;;
esac
echo "Extension ready in $bin"

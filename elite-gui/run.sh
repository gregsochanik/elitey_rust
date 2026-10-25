#!/bin/sh
# Build the Rust bindings, generate the Swift glue and run the macOS app.
set -eu
cd "$(dirname "$0")/.."

cargo build --release -p elite-ffi
generated=elite-gui/Sources/EliteGUI/Generated
rm -rf "$generated" && mkdir -p "$generated"
cargo run -q -p elite-ffi --bin uniffi-bindgen -- generate \
  --library target/release/libelite_ffi.dylib --language swift --out-dir "$generated"

include=elite-gui/Sources/elite_ffiFFI/include
mv "$generated/elite_ffiFFI.h" "$include/elite_ffiFFI.h"
mv "$generated/elite_ffiFFI.modulemap" "$include/module.modulemap"

# SwiftUI macros need the full Xcode toolchain, not just the command line tools
if [ -z "${DEVELOPER_DIR:-}" ] && [ -d /Applications/Xcode.app ]; then
  export DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer
fi

cd elite-gui
swift run EliteGUI

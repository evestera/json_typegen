#!/usr/bin/env bash

set -ex

DIR="$(cd "$(dirname $0)/.." && pwd)"

cd "$DIR/json_typegen_shared"
cargo build --target "$TARGET" --verbose
cargo test --target "$TARGET" --verbose

cd "$DIR/json_typegen_cli"
cargo build --target "$TARGET" --verbose
cargo test --target "$TARGET" --verbose

cd "$DIR/json_typegen_web"
cargo build --target "$TARGET" --verbose
cargo test --target "$TARGET" --verbose

cd "$DIR/json_typegen"
cargo build --target "$TARGET" --verbose
cargo test --target "$TARGET" --verbose

cd "$DIR/json_typegen_demo"
cargo run --target "$TARGET" --verbose

#!/usr/bin/env bash

set -ex

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$DIR/json_typegen_shared"
cargo build --verbose
cargo test --verbose

cd "$DIR/json_typegen_cli"
cargo build --verbose
cargo test --verbose

cd "$DIR/json_typegen_web"
cargo build --verbose
cargo test --verbose

cd "$DIR/json_typegen"
cargo build --verbose
cargo test --verbose

cd "$DIR/json_typegen_demo"
cargo run --verbose

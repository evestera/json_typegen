#!/usr/bin/env bash

set -ex

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$DIR/json_sample_shared"
cargo build --verbose
cargo test --verbose

cd "$DIR/json_sample_cli"
cargo build --verbose
cargo test --verbose

cd "$DIR/json_sample_derive"
cargo build --verbose
cargo test --verbose

cd "$DIR/json_sample_web"
cargo build --verbose
cargo test --verbose

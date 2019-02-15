#!/usr/bin/env bash

set -ex

build() {
    cargo build --package json_typegen_cli --target "$TARGET" --release --verbose
}

pack() {
    local temp_dir="$(mktemp -d)"
    local out_dir="$(pwd)/deployment"
    local package_name="${PROJECT_NAME}-${TRAVIS_TAG}-${TARGET}"
    local staging="$temp_dir/$package_name"

    mkdir -p "$staging"
    mkdir -p "$out_dir"

    cp "target/$TARGET/release/$PROJECT_NAME" "$staging/$PROJECT_NAME"

    (cd "$temp_dir" && tar czf "$out_dir/$package_name.tar.gz" "$package_name")
    rm -rf "$temp_dir"
}

build
pack

# CLI for generation of type definitions for Rust, Kotlin, TypeScript and JSON Schema from JSON

**Note**: For Rust types there is also a procedural macro interface to this code, which uses
the same underlying algorithm and code generation. See
[the repository](https://github.com/evestera/json_typegen) for details.


## Installation

```sh
cargo install json_typegen_cli
# installed binary is called json_typegen
# make sure ~/.cargo/bin is on your PATH
```

## Usage

To generate the Rust type `Point` in `point.rs` from a local sample, run:

```sh
json_typegen json_samples/point.json -o src/point.rs -n 'pub Point'
```

*Note: The output file (e.g. `src/point.rs`) will be overwritten if it exists.*

For an online sample, run:

```sh
json_typegen 'http://vestera.as/json_typegen/examples/point.json' -o src/point.rs -n 'pub Point'
```

The generated code assumes the availability of `serde` and `serde_derive`, so
make sure your `Cargo.toml` contains something like:

```toml
[dependencies]
serde = "1.0"
serde_derive = "1.0"
# Not required, but you probably also want:
serde_json = "1.0"
```

## Options and configurations

For help with the CLI itself run `json_typegen -h`. To configure visibility and
other options see [the general configuration documentation](../CONFIGURATION.md).

## Other languages

The CLI currently has rudimentary support for Kotlin, TypeScript and JSON Schema. Contributions for these and additions of others are welcome.

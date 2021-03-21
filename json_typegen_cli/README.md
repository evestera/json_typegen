# CLI for generation of type definitions for Rust, Kotlin, TypeScript and JSON Schema from JSON

**Note**: There is also a web interface, and for Rust types there is also a
procedural macro interface to this code, which uses
the same underlying algorithm and code generation. See
[the repository](https://github.com/evestera/json_typegen) for details
and [typegen.vestera.as](https://typegen.vestera.as) for the web interface.


## Installation

Install with `cargo`:

```sh
cargo install json_typegen_cli
# installed binary is called json_typegen
# make sure ~/.cargo/bin is on your PATH
```

Or download precompiled binaries from the
[GitHub releases page](https://github.com/evestera/json_typegen/releases).

## Usage

To generate the Rust type `Point` in `point.rs` from a local sample, run:

```sh
json_typegen json_samples/point.json -o src/point.rs -n Point
```

*Note: The output file (e.g. `src/point.rs`) will be overwritten if it exists.*

For an online sample, run:

```sh
json_typegen 'https://typegen.vestera.as/examples/point.json' -o src/point.rs -n Point
```

The generated code assumes the availability of `serde` and `serde_derive`, so
make sure your `Cargo.toml` contains something like:

```toml
[dependencies]
serde = "1.0"
serde_derive = "1.0"
# Not required for the types themselves, but you probably also want:
serde_json = "1.0"
```

## Options and configurations

For help with the CLI itself run `json_typegen -h`. To configure visibility and
other options see [the general configuration documentation](../CONFIGURATION.md).

## Other languages

You can output code for other languages with the `--output-mode` option. From `--help`:

```
    -O, --output-mode <output-mode>    What to output. [possible values: rust, typescript, typescript/typealias, kotlin,
                                       kotlin/jackson, kotlin/kotlinx, json_schema, shape]
```

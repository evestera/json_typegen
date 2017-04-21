# CLI for JSON code generation for Rust

**Note**: The main intended interface for this code generation is a procedural macro. See [its docs](https://docs.rs/crate/json_typegen) for details.


## Installation

```sh
cargo install json_typegen_cli
# installed binary is called json_typegen
```

Due to the fact that this tool uses `rustfmt` to make code look reasonable, compilation takes a while. If you are impatient, you can use the [web interface](http://vestera.as/json_typegen/) in the meantime.

## Usage

To generate the type `Point` in `point.rs` from a local sample, run:

```
json_typegen json_samples/point.json -o src/point.rs -n Point
```

Or for an online sample, run:

```
json_typegen 'http://vestera.as/json_typegen/examples/point.json' -o src/point.rs -n Point
```

The generated code assumes the availability of `serde` and `serde_derive`, so make sure your `Cargo.toml` contains something like:

```toml
[dependencies]
serde = "0.9"
serde_derive = "0.9"
# Not required, but you probably also want:
serde_json = "0.9"
```

And your crate root (i.e. `main.rs`) should contain at least:

```rust
extern crate serde;
#[macro_use]
extern crate serde_derive;
// Again, not required, but you probably also want:
extern crate serde_json;
```

# CLI for JSON code generation for Rust

**Note**: The main intended interface for this code generation is a procedural macro. See [the main readme](../README.md) for details.


## Installation

Since the crate is not on crates.io yet, this is a bit more cumbersome than it will be in the future. Due to using `rustfmt` to make code look reasonable, compilation also takes a while. If you are impatient, you can use the [web interface](../README.md#web-interface) in the meantime.

```
git clone https://github.com/evestera/json_sample
cd json_sample/json_sample_cli
cargo install
```


## Usage

To generate the type `Point` in `point.rs` from an online sample, run:

```
json_sample_cli 'http://vestera.as/json_sample/examples/point.json' -o src/point.rs -n Point
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

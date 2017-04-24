# JSON code generation tools for Rust

[![Travis Build Status](https://api.travis-ci.org/evestera/json_typegen.svg?branch=master)](https://travis-ci.org/evestera/json_typegen)
[![crates.io](https://img.shields.io/crates/v/json_typegen.svg)](https://crates.io/crates/json_typegen)
[![docs.rs](https://docs.rs/json_typegen/badge.svg)](https://docs.rs/json_typegen/)

WARNING: This project is still in early development and you should not rely on it outputting exactly the same code tomorrow as it does today. That said, feel free to try things out, and feedback/issues are very welcome.

This is a collection of tools for generating Rust types from JSON samples. It was inspired by and uses the same kind of inference algorithm as [F# Data](http://fsharp.github.io/FSharp.Data/). There are three interfaces to the code generation:

- [Procedural macro](#procedural-macro)
- [Command line interface](#command-line-interface)
- [Web interface](#web-interface)

## Procedural macro

The main interface to the code generation tools is a procedural macro `json_typegen!`. As an example, the below code generates code for the type Point, including derives for serialization and deserialization (using [serde_derive](https://crates.io/crates/serde_derive)).

```rust
#[macro_use]
extern crate json_typegen;
extern crate serde_json;

json_typegen!("Point", r#"{ "x": 1, "y": 2 }"#);

fn main() {
    let mut p: Point = serde_json::from_str(r#"{ "x": 3, "y": 5 }"#).unwrap();
    println!("deserialized = {:?}", p);
    p.x = 4;
    let serialized = serde_json::to_string(&p).unwrap();
    println!("serialized = {}", serialized);
}
```

```toml
[dependencies]
serde = "0.9"
serde_json = "0.9"
json_typegen = { git = "https://github.com/evestera/json_typegen/" }
```

The sample json can also come from local or remote files, like so:

```rust
json_typegen!("Point", "json_samples/point.json");

json_typegen!("Point", "http://example.com/someapi/point.json");
```

### Conditional compilation

To avoid incurring the cost of a http request per sample used for every build you can use conditional compilation to only check against remote samples when desired:

```rust
#[cfg(not(feature = "online-samples"))]
json_typegen!("Point", r#"{ "x": 1, "y": 2 }"#);
#[cfg(feature = "online-samples")]
json_typegen!("Point", "http://vestera.as/json_typegen/examples/point.json");
```

And in Cargo.toml:
```toml
[features]
online-samples = []
```

You can then verify that remote samples match your expectations in e.g. CI builds as follows:

```sh
cargo check --features "online-samples"
```


## Command line interface

The crate `json_typegen_cli` provides a CLI to the same code generation as the procedural macro uses internally. This provides a useful migration path if you at some point need to customize the generated code beyond what is practical through macro arguments.

For details on usage see [its readme](json_typegen_cli/README.md).


## Web interface

For simple testing and one-time use there is also a web interface (in `json_typegen_web`). An instance of this interface is currently hosted at <http://vestera.as/json_typegen>

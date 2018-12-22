# json_typegen - Rust types from JSON samples

[![Travis Build Status](https://api.travis-ci.org/evestera/json_typegen.svg?branch=master)](https://travis-ci.org/evestera/json_typegen)
[![crates.io](https://img.shields.io/crates/v/json_typegen.svg)](https://crates.io/crates/json_typegen)
[![docs.rs](https://docs.rs/json_typegen/badge.svg)](https://docs.rs/json_typegen/)

*json_typegen* is a collection of tools for generating Rust types from JSON samples, built on top of [serde]. I.e. you give it some JSON, and it gives you the type definitions necessary to use that JSON in a Rust program. If you are familiar with [F#], the procedural macro `json_typegen!` works as a [type provider] for JSON in Rust. It was inspired by and uses the same kind of inference algorithm as [F# Data].

[serde]: https://serde.rs/
[F# Data]: http://fsharp.github.io/FSharp.Data/
[F#]: http://fsharp.org/
[type provider]: https://docs.microsoft.com/en-us/dotnet/fsharp/tutorials/type-providers/

There are three interfaces to this code generation logic:

- [Procedural macro](#procedural-macro)
- [Command line interface](#command-line-interface)
- [Web interface](#web-interface)

The CLI and web interface also produces types for other languages than Rust. Namely Kotlin, TypeScript and JSON Schemas.

## Procedural macro

The first interface to the code generation tools is a procedural macro `json_typegen!`. As an example, the below code generates code for the type `Point`.

```rust
use json_typegen::json_typegen;

json_typegen!("pub Point", r#"{ "x": 1, "y": 2 }"#);

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
serde = "1.0"
serde_derive = "1.0"
serde_json = "1.0"
json_typegen = "0.3"
```

The sample json can also come from local or remote files:

```rust
json_typegen!("pub Point", "json_samples/point.json");
json_typegen!("pub Point", "http://example.com/someapi/point.json");
```

The code generation can also be customized:

```rust
json_typegen!("pub Point", "http://example.com/someapi/point.json", {
    use_default_for_missing_fields,
    "/foo/bar": {
        use_type: "map"
    }
});
```

For the details, see [the relevant documentation](CONFIGURATION.md).

### Conditional compilation

To avoid incurring the cost of a HTTP request per sample used for every build you can use conditional compilation to only check against remote samples when desired:

```rust
#[cfg(not(feature = "online-samples"))]
json_typegen!("pub Point", r#"{ "x": 1, "y": 2 }"#);
#[cfg(feature = "online-samples")]
json_typegen!("pub Point", "http://vestera.as/json_typegen/examples/point.json");
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

## Creating your own type provider crate

Both procedural macros and the shape inference algorithm are actually very simple. To learn/copy the algorithm you can look at [this stripped-down version](https://github.com/evestera/thesis/tree/master/code/shape_inference)(< 200 lines).

## License

This project is dual licensed, under either the Apache 2.0 or the MIT license, at your option.

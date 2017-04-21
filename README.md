# JSON code generation tools for Rust

**WARNING**: This project is still in early development and should not be relied upon. However, feedback and issues are very welcome.

This is a collection of tools for generating structs from JSON samples.

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

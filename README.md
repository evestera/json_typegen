# JSON code generation tools for Rust

**WARNING**: This project is still in early development and should not be relied upon. However, feedback and issues are very welcome.

This is a collection of tools for generating structs from JSON samples.

## Procedural macro

The main interface to the code generation tools is a procedural macro `json_provider!`. As an example, the below code generates code for the type Point, including derives for serialization and deserialization (using [serde_derive](https://crates.io/crates/serde_derive)).

```rust
extern crate serde_json;
#[macro_use]
extern crate json_provider;

json_provider!("Point", r#"{ "x": 1, "y": 2 }"#);

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
json_provider = "*"
```

The sample json can also come from local or remote files, like so:

```rust
json_provider!("Point", "json_samples/point.json");

json_provider!("Point", "http://example.com/someapi/point.json");
```

## Command line interface

The crate `json_sample_cli` provides a CLI to the same code generation.


## Web interface

For simple testing and one-time use there is also a web interface (in `json_sample_web`). An instance of this interface is currently hosted at <http://vestera.as/json_sample>

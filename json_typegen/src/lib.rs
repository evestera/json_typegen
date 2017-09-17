//! This crate provides the procedural macro `json_typegen!` which creates Rust
//! types from JSON samples. As an example, the below code generates code for
//! the type Point, including derives for serialization and deserialization
//! (using [serde_derive](https://crates.io/crates/serde_derive)).
//!
//! ```rust
//! #[macro_use]
//! extern crate json_typegen;
//! extern crate serde_json;
//!
//! json_typegen!("Point", r#"{ "x": 1, "y": 2 }"#);
//!
//! fn main() {
//!     let mut p: Point = serde_json::from_str(r#"{ "x": 3, "y": 5 }"#).unwrap();
//!     println!("deserialized = {:?}", p);
//!     p.x = 4;
//!     let serialized = serde_json::to_string(&p).unwrap();
//!     println!("serialized = {}", serialized);
//! }
//! ```
//!
//! ```toml
//! [dependencies]
//! serde = "0.9"
//! serde_json = "0.9"
//! json_typegen = "0.1"
//! ```
//!
//! The sample json can also come from local or remote files, like so:
//!
//! ```rust,ignore
//! json_typegen!("Point", "json_samples/point.json");
//!
//! json_typegen!("Point", "http://example.com/someapi/point.json");
//! ```
//!
//! ### Conditional compilation
//!
//! To avoid incurring the cost of a http request per sample used for every
//! build you can use conditional compilation to only check against remote
//! samples when desired:
//!
//! ```rust,ignore
//! #[cfg(not(feature = "online-samples"))]
//! json_typegen!("Point", r#"{ "x": 1, "y": 2 }"#);
//! #[cfg(feature = "online-samples")]
//! json_typegen!("Point", "http://vestera.as/json_typegen/examples/point.json");
//! ```
//!
//! And in Cargo.toml:
//! ```toml
//! [features]
//! online-samples = []
//! ```
//!
//! You can then verify that remote samples match your expectations in e.g. CI
//! builds as follows:
//!
//! ```sh
//! cargo check --features "online-samples"
//! ```

#[allow(unused_imports)]
#[macro_use]
extern crate json_typegen_derive;
#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

pub use json_typegen_derive::*;
pub use serde_derive::*;

/// The main point of this crate
/// See root documentation
#[macro_export]
macro_rules! json_typegen {
    ($($input:tt)*) => {
        #[derive(json_types)]
        #[allow(unused)]
        enum JsonTypegenPlaceholder {
            Input = (stringify!($($input)*), 0).1
        }
    };
}

//! This crate provides the procedural macro `json_typegen!` which creates Rust
//! types from JSON samples. As an example, the below code generates code for
//! the type Point, including derives for serialization and deserialization
//! (using [serde_derive](https://crates.io/crates/serde_derive)).
//!
//! ```rust
//! use json_typegen::json_typegen;
//!
//! json_typegen!("Point", r#"{ "x": 1, "y": 2 }"#);
//!
//! let mut p: Point = serde_json::from_str(r#"{ "x": 3, "y": 5 }"#).unwrap();
//! println!("deserialized = {:?}", p);
//! p.x = 4;
//! let serialized = serde_json::to_string(&p).unwrap();
//! println!("serialized = {}", serialized);
//! ```
//!
//! ```toml
//! [dependencies]
//! serde = "1.0"
//! serde_derive = "1.0"
//! serde_json = "1.0"
//! json_typegen = "0.7"
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
//! json_typegen!("Point", "https://typegen.vestera.as/examples/point.json");
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

extern crate proc_macro;

use json_typegen_shared::codegen_from_macro_input;
use json_typegen_shared::internal_util::display_error_with_causes;

/// Generate serde-compatible types from JSON
///
/// `json_typegen!(<type name>, <sample source>, <options?>)`
#[proc_macro]
pub fn json_typegen(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match codegen_from_macro_input(&input.to_string()) {
        Ok(code) => code,
        Err(e) => {
            let message = display_error_with_causes(&e);
            format!(r##"compile_error!(r#"{}"#);"##, message)
        }
    }
    .parse()
    .unwrap()
}

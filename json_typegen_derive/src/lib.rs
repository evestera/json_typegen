extern crate proc_macro;
extern crate json_typegen_shared;

use json_typegen_shared::{codegen_from_macro_input};
use proc_macro::TokenStream;

#[proc_macro_derive(json_types)]
pub fn derive_json_typegen(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let macro_input = extract_from_proc_macro_hack(&source);
    codegen_from_macro_input(macro_input).parse().unwrap()
}

// Adapted from:
// https://github.com/dtolnay/proc-macro-hack/blob/eae65f5aa229137576fd42f25241c4186c276cdf/src/lib.rs#L365
fn extract_from_proc_macro_hack(source: &str) -> &str {
    let source = source.trim();

    let prefix = "#[allow(unused)]\nenum JsonTypegenPlaceholder {";
    let suffix = "}";
    assert!(source.starts_with(prefix));
    assert!(source.ends_with(suffix));
    let source = &source[prefix.len() .. source.len() - suffix.len()].trim();

    let prefix = "Input =";
    let suffix = "0).1,";
    assert!(source.starts_with(prefix));
    assert!(source.ends_with(suffix));
    let source = &source[prefix.len() .. source.len() - suffix.len()].trim();

    let prefix = "(stringify!(";
    let suffix = "),";
    assert!(source.starts_with(prefix));
    assert!(source.ends_with(suffix));
    let source = &source[prefix.len() .. source.len() - suffix.len()].trim();

    source
}

extern crate swagger_shared;

use swagger_shared::{codegen_from_spec, SpecSource};

use std::env;

fn main() {
    match env::args().skip(1).next() {
        Some(str) => {
            let tokens = codegen_from_spec("Foobar", SpecSource::File(&str));
            println!("{}", tokens.unwrap());
        },
        None => {
            println!("Usage: rs-swagger <json spec file>");
        }
    }
}
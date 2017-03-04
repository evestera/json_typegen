extern crate json_sample_shared;
extern crate rustfmt;

use std::env;
use json_sample_shared::{codegen_from_spec, SpecSource};
use rustfmt::config::{Config, WriteMode};

fn main() {
    // TODO: Add proper arg parsing and more configuration
    // - At least: Input, output, name
    match env::args().skip(1).next() {
        Some(str) => {
            let tokens = codegen_from_spec("json_sampleApi", SpecSource::File(&str)).unwrap();
            let mut code = String::from("pub struct json_sampleApi;");
            code += tokens.as_str();

            let input = rustfmt::Input::Text(code);
            let mut output = std::io::stdout();
            let mut config = Config::default();
            config.write_mode = WriteMode::Plain;
            rustfmt::format_input(input, &config, Some(&mut output)).unwrap();
        }
        None => {
            println!("Usage: rs-json_sample <json spec file>");
        }
    }
}

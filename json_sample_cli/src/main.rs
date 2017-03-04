extern crate json_sample_shared;
extern crate rustfmt;

use std::env;
use json_sample_shared::{codegen_from_sample, SampleSource};
use rustfmt::config::{Config, WriteMode};

fn main() {
    // TODO: Add proper arg parsing and more configuration
    // - At least: Input, output, name
    match env::args().skip(1).next() {
        Some(str) => {
            let tokens = codegen_from_sample("Sample", SampleSource::File(&str)).unwrap();
            let input = rustfmt::Input::Text(tokens.as_str());
            let mut output = std::io::stdout();
            let mut config = Config::default();
            config.write_mode = WriteMode::Plain;
            rustfmt::format_input(input, &config, Some(&mut output)).unwrap();
        }
        None => {
            println!("Usage: json_sample_cli <json file>");
        }
    }
}

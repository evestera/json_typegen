extern crate json_typegen_shared;
extern crate rustfmt;
extern crate clap;

use json_typegen_shared::{codegen, infer_source_type, Options};
use rustfmt::config::{Config, WriteMode};
use clap::{Arg, App};

fn main() {
    let matches = App::new("JSON code generation CLI")
                      .version("0.1.0")
                      .about("Generate Rust types from JSON samples")
                      .arg(Arg::with_name("input")
                           .help("The input JSON to generate types from. Can be a file or a http/https URL")
                           .takes_value(true)
                           .required(true))
                      .arg(Arg::with_name("output")
                           .short("o")
                           .long("output")
                           .help("What file to write the output to. Default: standard output.")
                           .takes_value(true))
                      .arg(Arg::with_name("name")
                           .short("n")
                           .long("name")
                           .help("Name for the root generated type. Default: Root.")
                           .takes_value(true))
                      .get_matches();

    let source = infer_source_type(matches.value_of("input").unwrap());
    let name = matches.value_of("name").unwrap_or("Root");
    let tokens = codegen(&name, &source, Options::default()).unwrap();
    let input = rustfmt::Input::Text(String::from(tokens.as_str()));
    let mut config = Config::default();
    config.write_mode = WriteMode::Plain;
    if let Some(output) = matches.value_of("output") {
        let mut file = std::fs::File::create(output).unwrap();
        rustfmt::format_input(input, &config, Some(&mut file)).unwrap();
    } else {
        rustfmt::format_input(input, &config, Some(&mut std::io::stdout())).unwrap();
    }
}

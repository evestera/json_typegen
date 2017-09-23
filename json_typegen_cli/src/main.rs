extern crate json_typegen_shared;
extern crate clap;

use json_typegen_shared::codegen_from_macro;
use clap::{Arg, App};

fn main() {
    let matches = App::new("JSON code generation CLI")
                      .version("0.1.0")
                      .about("Generate Rust types from JSON samples")
                      .arg(Arg::with_name("input")
                           .help("The input macro to generate types from.")
                           .takes_value(true)
                           .required(true))
                      .get_matches();

    let source = matches.value_of("input").unwrap();
    let code = codegen_from_macro(&source).unwrap();
    print!("{}", code);
}

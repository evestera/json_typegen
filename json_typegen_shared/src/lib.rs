#[macro_use]
extern crate quote;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate linked_hash_map;
extern crate inflector;
extern crate regex;
extern crate syn;
#[macro_use]
extern crate synom;

use std::fs::File;
use serde_json::Value;
use quote::Tokens;
use regex::Regex;

#[macro_use]
mod util;
mod inference;
mod generation;
mod hints;
mod options;
mod parse;

use hints::Hints;
pub use options::Options;

mod errors {
    error_chain! {
        types {
            Error, ErrorKind, ResultExt, Result;
        }

        links {
        }

        foreign_links {
            ReqwestError(::reqwest::Error);
            JsonError(::serde_json::Error);
            IoError(::std::io::Error);
        }

        errors {
            MissingSource {
                description("No source for sample specified")
            }
            ExistingType(t: String) {
                display("No code generated, JSON matches existing type {}", t)
            }
        }
    }
}

pub use errors::*;

pub enum SampleSource<'a> {
    Url(&'a str),
    File(&'a str),
    Text(&'a str),
}

pub fn from_str_with_defaults(name: &str, json: &str) -> Result<Tokens> {
    codegen(name, &SampleSource::Text(json), Options::default())
}

pub fn codegen_from_macro(input: &str) -> String {
    let macro_input = parse::full_macro(input).unwrap();

    codegen(
        &macro_input.name,
        &infer_source_type(&macro_input.sample_source),
        macro_input.options,
    ).unwrap()
        .to_string()
}

pub fn codegen_from_macro_input(input: &str) -> String {
    let macro_input = parse::macro_input(input).unwrap();

    codegen(
        &macro_input.name,
        &infer_source_type(&macro_input.sample_source),
        macro_input.options,
    ).unwrap()
        .to_string()
}

pub fn codegen(name: &str, source: &SampleSource, mut options: Options) -> Result<Tokens> {
    let sample = get_and_parse_sample(source)?;
    let name = handle_pub_in_name(name, &mut options);

    let mut hints_vec = Vec::new();
    std::mem::swap(&mut options.hints, &mut hints_vec);

    let mut hints = Hints::new();
    for &(ref pointer, ref hint) in hints_vec.iter() {
        hints.add(&pointer, &hint);
    }

    let shape = inference::value_to_shape(&sample, &hints);

    let generated_code = if options.runnable {
        generation::shape_to_example_program(name, &shape, options)
    } else {
        let (name, defs) = generation::shape_to_type_defs(name, &shape, options);
        defs.ok_or(Error::from(ErrorKind::ExistingType(name.to_string())))?
    };

    Ok(generated_code)
}

/// Parse "names" like `pub(crate) Foo` into a name and a visibility option
fn handle_pub_in_name<'a>(name: &'a str, options: &mut Options) -> &'a str {
    lazy_static! {
        static ref PUB_RE: Regex =
            Regex::new(r"(?x)
                pub ( \( (?P<restriction> [^)]+ ) \) )?
                \s+
                (?P<name> .+ )
            ").unwrap();
    }
    match PUB_RE.captures(name) {
        Some(captures) => {
            options.type_visibility = match captures.name("restriction") {
                Some(restriction) => format!("pub({})", restriction.as_str()),
                None => "pub".into(),
            };
            captures.name("name").unwrap().as_str()
        }
        None => {
            // If there is no visibility specified here, we want to use whatever is set elsewhere
            name
        }
    }
}

pub fn infer_source_type(s: &str) -> SampleSource {
    let s = s.trim();
    if s.starts_with('{') || s.starts_with('[') {
        return SampleSource::Text(s);
    }
    if s.starts_with("http://") || s.starts_with("https://") {
        return SampleSource::Url(s);
    }
    SampleSource::File(s)
}

fn get_and_parse_sample(source: &SampleSource) -> Result<Value> {
    let parse_result = match *source {
        SampleSource::Url(url) => serde_json::de::from_reader(reqwest::get(url)?),
        SampleSource::File(path) => serde_json::de::from_reader(File::open(path)?),
        SampleSource::Text(text) => serde_json::from_str(text),
    };
    Ok(parse_result.chain_err(|| "Unable to parse JSON sample")?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_handle_pub_in_name() {
        let mut options = Options::default();
        let name = handle_pub_in_name("Foo", &mut options);
        assert_eq!(name, "Foo");
        assert_eq!(options.type_visibility, Options::default().type_visibility);
        let name = handle_pub_in_name("pub Foo", &mut options);
        assert_eq!(name, "Foo");
        assert_eq!(options.type_visibility, "pub".to_string());
        let name = handle_pub_in_name("pub(crate) Foo Bar", &mut options);
        assert_eq!(name, "Foo Bar");
        assert_eq!(options.type_visibility, "pub(crate)".to_string());
        let name = handle_pub_in_name("pub(some::path) Foo", &mut options);
        assert_eq!(name, "Foo");
        assert_eq!(options.type_visibility, "pub(some::path)".to_string());
    }
}

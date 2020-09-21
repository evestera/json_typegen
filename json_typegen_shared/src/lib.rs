//! Note: This crate is considered internal API of the `json_typegen` tools. If you want to use this
//! crate directly, and thus care about its stability, please
//! [open an issue](https://github.com/evestera/json_typegen/issues/new) to let me know.

#[macro_use]
extern crate error_chain;

#[cfg(feature = "local-samples")]
use std::fs::File;

use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

mod generation;
mod hints;
mod inference;
mod options;
pub mod parse;
mod shape;
mod util;

use crate::hints::Hints;
pub use crate::options::Options;
pub use crate::options::OutputMode;

mod errors {
    error_chain! {
        types {
            JTError, ErrorKind, ResultExt;
        }

        links {
        }

        foreign_links {
            ReqwestError(::reqwest::Error) #[cfg(feature = "remote-samples")];
            IoError(::std::io::Error) #[cfg(feature = "local-samples")];
            JsonError(::serde_json::Error);
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

pub use crate::errors::*;

enum SampleSource<'a> {
    Url(&'a str),
    File(&'a str),
    Text(&'a str),
}

/// Generate code from a `json_typegen` macro invocation
pub fn codegen_from_macro(input: &str) -> Result<String, JTError> {
    let macro_input = parse::full_macro(input)?;

    codegen(
        &macro_input.name,
        &macro_input.sample_source,
        macro_input.options,
    )
}

/// Generate code from the arguments to a `json_typegen` macro invocation
pub fn codegen_from_macro_input(input: &str) -> Result<String, JTError> {
    let macro_input = parse::macro_input(input)?;

    codegen(
        &macro_input.name,
        &macro_input.sample_source,
        macro_input.options,
    )
}

/// The main code generation function for `json_typegen`
pub fn codegen(name: &str, input: &str, mut options: Options) -> Result<String, JTError> {
    let source = infer_source_type(input);
    let sample = get_and_parse_sample(&source)?;
    let name = handle_pub_in_name(name, &mut options);

    let mut hints_vec = Vec::new();
    std::mem::swap(&mut options.hints, &mut hints_vec);

    let mut hints = Hints::new();
    for (pointer, hint) in hints_vec.iter() {
        hints.add(pointer, hint);
    }

    let shape = inference::value_to_shape(&sample, &hints);

    let mut generated_code = if options.runnable {
        generation::rust::rust_program(name, &shape, options)
    } else {
        let (name, defs) = match options.output_mode {
            OutputMode::Rust => generation::rust::rust_types(name, &shape, options),
            OutputMode::JsonSchema => generation::json_schema::json_schema(name, &shape, options),
            OutputMode::Kotlin => generation::kotlin::kotlin_types(name, &shape, options),
            OutputMode::Shape => generation::shape::shape_string(name, &shape, options),
            OutputMode::Typescript => {
                generation::typescript::typescript_types(name, &shape, options)
            }
        };
        defs.ok_or_else(|| JTError::from(ErrorKind::ExistingType(name.to_string())))?
    };

    if !generated_code.ends_with("\n") {
        generated_code.push('\n');
    }

    Ok(generated_code)
}

/// Parse "names" like `pub(crate) Foo` into a name and a visibility option
fn handle_pub_in_name<'a>(name: &'a str, options: &mut Options) -> &'a str {
    lazy_static! {
        static ref PUB_RE: Regex = Regex::new(
            r"(?x)
                pub ( \( (?P<restriction> [^)]+ ) \) )?
                \s+
                (?P<name> .+ )
            "
        )
        .unwrap();
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

fn infer_source_type(s: &str) -> SampleSource {
    let s = s.trim();
    if s.starts_with('{') || s.starts_with('[') {
        return SampleSource::Text(s);
    }
    if s.starts_with("http://") || s.starts_with("https://") {
        return SampleSource::Url(s);
    }
    SampleSource::File(s)
}

fn get_and_parse_sample(source: &SampleSource) -> Result<Value, JTError> {
    let parse_result = match *source {
        #[cfg(feature = "remote-samples")]
        SampleSource::Url(url) => serde_json::de::from_reader(reqwest::get(url)?),
        #[cfg(not(feature = "remote-samples"))]
        SampleSource::Url(_) => {
            return Err("Remote samples disabled".into());
        }

        #[cfg(feature = "local-samples")]
        SampleSource::File(path) => serde_json::de::from_reader(File::open(path)?),
        #[cfg(not(feature = "local-samples"))]
        SampleSource::File(_) => {
            return Err("Local samples disabled".into());
        }

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

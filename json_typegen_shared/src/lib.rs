//! [json_typegen](https://typegen.vestera.as/) as just a library,
//! for use in build scripts and other crates.
//! If you want an actual interface, like a website, CLI or procedural macro, check the repo:
//! [github.com/evestera/json_typegen](https://github.com/evestera/json_typegen)
//!
//! Note: This crate is to a certain extent considered internal API of the `json_typegen` tools.
//! If you want to use this crate directly, be prepared for breaking changes to happen, and consider
//! [opening an issue](https://github.com/evestera/json_typegen/issues/new)
//! to let me know what you are using. (Breaking changes may still happen,
//! but then I'll at least try to keep your use-case in mind if possible.
//! This has happened enough by now that there are parts I already consider public API.)

use thiserror::Error;

mod generation;
mod hints;
mod inference;
mod options;
#[cfg(feature = "option-parsing")]
pub mod parse;
#[cfg(feature = "progress")]
mod progress;
mod shape;
mod to_singular;
mod util;

use crate::hints::Hints;
use crate::inference::shape_from_json;
pub use crate::options::{ImportStyle, Options, OutputMode, StringTransform};
pub use crate::shape::Shape;

/// The errors that json_typegen_shared may produce
///
/// No stability guarantees are made with for this type
/// except that it is a type that implements `std::error::Error`
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum JTError {
    #[cfg(feature = "remote-samples")]
    #[error("An error occurred while fetching JSON")]
    SampleFetchingError(#[from] reqwest::Error),
    #[cfg(feature = "local-samples")]
    #[error("An error occurred while reading JSON from file")]
    SampleReadingError(#[from] std::io::Error),
    #[error("An error occurred while parsing JSON")]
    JsonParsingError(#[from] inference::JsonInputErr),
    #[error("An error occurred while parsing a macro or macro input: {0}")]
    MacroParsingError(String),
}

/// Utilities exposed only to be available inside the `json_typegen` workspace. Internal API.
pub mod internal_util {
    pub fn display_error_with_causes(error: &dyn std::error::Error) -> String {
        let mut message = format!("{}", error);
        let mut err = error;
        while let Some(source) = err.source() {
            message += &format!("\n  Caused by: {}", source);
            err = source;
        }
        message
    }
}

enum SampleSource<'a> {
    #[cfg(feature = "remote-samples")]
    Url(&'a str),
    #[cfg(feature = "local-samples")]
    File(&'a str),
    Text(&'a str),
}

#[cfg(feature = "option-parsing")]
/// Generate code from a `json_typegen` macro invocation
pub fn codegen_from_macro(input: &str) -> Result<String, JTError> {
    let macro_input = parse::full_macro(input).map_err(JTError::MacroParsingError)?;

    codegen(
        &macro_input.name,
        &macro_input.sample_source,
        macro_input.options,
    )
}

#[cfg(feature = "option-parsing")]
/// Generate code from the arguments to a `json_typegen` macro invocation
pub fn codegen_from_macro_input(input: &str) -> Result<String, JTError> {
    let macro_input = parse::macro_input(input).map_err(JTError::MacroParsingError)?;

    codegen(
        &macro_input.name,
        &macro_input.sample_source,
        macro_input.options,
    )
}

/// The main code generation function for `json_typegen`
pub fn codegen(name: &str, input: &str, mut options: Options) -> Result<String, JTError> {
    let source = infer_source_type(input);
    let name = handle_pub_in_name(name, &mut options);

    let mut hints_vec = Vec::new();
    std::mem::swap(&mut options.hints, &mut hints_vec);

    let mut hints = Hints::new();
    for (pointer, hint) in hints_vec.iter() {
        hints.add(pointer, hint);
    }

    let shape = infer_from_sample(&source, &options, &hints)?;

    codegen_from_shape(name, &shape, options)
}

/// Just code generation, no inference
pub fn codegen_from_shape(name: &str, shape: &Shape, options: Options) -> Result<String, JTError> {
    let mut generated_code = match options.output_mode {
        OutputMode::Rust => generation::rust::rust_types(name, shape, options),
        OutputMode::JsonSchema => generation::json_schema::json_schema(name, shape, options),
        OutputMode::KotlinJackson | OutputMode::KotlinKotlinx => {
            generation::kotlin::kotlin_types(name, shape, options)
        }
        OutputMode::Shape => generation::shape::shape_string(name, shape, options),
        OutputMode::Typescript => generation::typescript::typescript_types(name, shape, options),
        OutputMode::TypescriptTypeAlias => {
            generation::typescript_type_alias::typescript_type_alias(name, shape, options)
        }
    };

    // Ensure generated code ends with exactly one newline
    generated_code.truncate(generated_code.trim_end().len());
    generated_code.push('\n');

    Ok(generated_code)
}

/// Parse "names" like `pub(crate) Foo` into a name and a visibility option
fn handle_pub_in_name<'a>(name: &'a str, options: &mut Options) -> &'a str {
    if let Some(suffix) = name.strip_prefix("pub ") {
        options.type_visibility = "pub".to_string();
        return suffix;
    }
    if name.starts_with("pub(") {
        if let Some((visibility, rest)) = name.split_once(") ") {
            options.type_visibility = format!("{})", visibility);
            return rest;
        }
    }
    name
}

fn infer_source_type(s: &str) -> SampleSource {
    let s = s.trim();
    if s.starts_with('{') || s.starts_with('[') {
        return SampleSource::Text(s);
    }
    #[cfg(feature = "remote-samples")]
    if s.starts_with("http://") || s.starts_with("https://") {
        return SampleSource::Url(s);
    }
    #[cfg(feature = "local-samples")]
    return SampleSource::File(s);
    #[cfg(not(feature = "local-samples"))]
    return SampleSource::Text(s);
}

fn infer_from_sample(
    source: &SampleSource,
    options: &Options,
    hints: &Hints,
) -> Result<Shape, JTError> {
    let parse_result = match *source {
        #[cfg(feature = "remote-samples")]
        SampleSource::Url(url) => {
            shape_from_json(reqwest::get(url)?.error_for_status()?, options, hints)
        }

        #[cfg(all(feature = "local-samples", feature = "progress"))]
        SampleSource::File(path) => shape_from_json(
            crate::progress::FileWithProgress::open(path)?,
            options,
            hints,
        ),
        #[cfg(all(feature = "local-samples", not(feature = "progress")))]
        SampleSource::File(path) => shape_from_json(std::fs::File::open(path)?, options, hints),

        SampleSource::Text(text) => shape_from_json(text.as_bytes(), options, hints),
    };
    Ok(parse_result?)
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

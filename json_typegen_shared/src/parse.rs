//! Functions for parsing `json_typegen` macro invocations and their arguments

use syn;
use syn::parse::{boolean, ident, string};
use synom::{alt, call, named, punct, IResult};

use crate::hints::Hint;
use crate::options::{Options, OutputMode, StringTransform};

#[derive(PartialEq, Debug)]
pub struct MacroInput {
    pub name: String,
    pub sample_source: String,
    pub options: Options,
}

macro_rules! fail {
    ($base:expr, $input:expr) => {
        return Err(format!(
            "{}, but remaining input was '{}'",
            $base,
            $input.trim()
        ));
    };
}

named!(string_or_ident -> String,
    alt!(
        ident => { |ident: syn::Ident| ident.to_string() }
        |
        string => { |lit: syn::StrLit| lit.value }
    )
);

named!(comma_or_closing_brace -> &str,
    alt!(punct!(",") | punct!("}"))
);

/// Parses a full `json_typegen` macro invocation. E.g. something like
/// `json_typegen!("Foo", "http://example.com/sample.json", { deny_unknown_fields });`
pub fn full_macro(input: &str) -> Result<MacroInput, String> {
    let input = input.trim();

    let prefix = "json_typegen!(";
    if !input.starts_with(prefix) {
        fail!("Unable to parse macro. Expected 'json_typegen!('", input)
    }

    let suffix = ");";
    if !input.ends_with(suffix) {
        fail!("Unable to parse macro. Expected it to end with ');'", input)
    }

    let input = &input[prefix.len()..input.len() - suffix.len()].trim();

    macro_input(input)
}

/// Parses the arguments to a `json_typegen` macro invocation. E.g. something like
/// `"Foo", "http://example.com/sample.json", { deny_unknown_fields }`
pub fn macro_input(input: &str) -> Result<MacroInput, String> {
    let (input, name) = match string(input) {
        IResult::Done(input, lit) => (input, lit.value),
        IResult::Error => fail!("First argument must be a string literal", input),
    };

    let input = skip(input, ",", "Expected a comma after first argument")?;

    let (input, sample_source) = match string(input) {
        IResult::Done(input, lit) => (input, lit.value),
        IResult::Error => fail!("Second argument must be a string literal", input),
    };

    if input.trim().is_empty() {
        return Ok(MacroInput {
            name,
            sample_source,
            options: Options::default(),
        });
    }

    let input = skip(
        input,
        ",",
        "Expected a comma or end of input after second argument",
    )?;

    let options = match string(input) {
        IResult::Done(_, lit) => options(&lit.value)?,
        IResult::Error => options(input)?,
    };

    Ok(MacroInput {
        name,
        sample_source,
        options,
    })
}

/// Parses the options block of a `json_typegen` macro invocation. E.g. something like:
/// `{ deny_unknown_fields }`
pub fn options(input: &str) -> Result<Options, String> {
    let mut options = Options::default();

    let input_after_block = block(input, |remaining, option_name| match option_name.as_ref() {
        "output_mode" => string_option(remaining, "output_mode", |val| {
            options.output_mode = OutputMode::parse(&val).unwrap_or(OutputMode::Rust);
        }),
        "derives" => string_option(remaining, "derives", |val| {
            options.derives = val;
        }),
        "property_name_format" => string_option(remaining, "rename_all", |val| {
            options.property_name_format = StringTransform::parse(&val)
        }),
        "field_visibility" => string_option(remaining, "field_visibility", |val| {
            options.field_visibility = Some(val);
        }),
        "deny_unknown_fields" => boolean_option(remaining, "deny_unknown_fields", |val| {
            options.deny_unknown_fields = val;
        }),
        "use_default_for_missing_fields" => {
            boolean_option(remaining, "use_default_for_missing_fields", |val| {
                options.use_default_for_missing_fields = val;
            })
        }
        "allow_option_vec" => boolean_option(remaining, "allow_option_vec", |val| {
            options.allow_option_vec = val;
        }),
        "type_alias_extant_types" => boolean_option(remaining, "type_alias_extant_types", |val| {
            options.type_alias_extant_types = val;
        }),
        key if key.is_empty() || key.starts_with('/') => {
            let (rem, hints) = pointer_block(remaining)?;
            for hint in hints {
                options.hints.push((key.to_string(), hint));
            }
            Ok(rem)
        }
        _ => Err(format!("Unknown option: {}", option_name)),
    })?;

    if !input_after_block.trim().is_empty() {
        fail!("Expected no further tokens after options block", input);
    }

    Ok(options)
}

fn pointer_block(input: &str) -> Result<(&str, Vec<Hint>), String> {
    let mut hints = Vec::new();

    let input = skip_colon(input)?;

    let input_after_block = block(input, |input_after_key, key| match key.as_ref() {
        "use_type" => string_option(input_after_key, "use_type", |val| {
            let hint = match val.as_ref() {
                "map" => Hint::default_map(),
                _ => Hint::opaque_type(val),
            };
            hints.push(hint);
        }),
        "type_name" => string_option(input_after_key, "type_name", |val| {
            hints.push(Hint::type_name(val));
        }),
        _ => Err(format!("Unknown option: {}", key)),
    })?;

    Ok((input_after_block, hints))
}

fn string_option<'a, F: FnMut(String)>(
    input: &'a str,
    name: &'static str,
    mut consumer: F,
) -> Result<&'a str, String> {
    let input = skip_colon(input)?;

    match string(input) {
        IResult::Done(rem, lit) => {
            consumer(lit.value);
            Ok(rem)
        }
        IResult::Error => fail!(
            format!("The argument to '{}' has to be a string literal", name),
            input
        ),
    }
}

fn boolean_option<'a, F: FnMut(bool)>(
    input: &'a str,
    name: &'static str,
    mut consumer: F,
) -> Result<&'a str, String> {
    // interpret { foo, bar } as { foo: true, bar: true }
    if let IResult::Done(_, _) = comma_or_closing_brace(input) {
        consumer(true);
        return Ok(input);
    }

    let input = skip_colon(input)?;

    match boolean(input) {
        IResult::Done(rem, val) => {
            consumer(val);
            Ok(rem)
        }
        IResult::Error => fail!(
            format!("The argument to '{}' has to be a boolean literal", name),
            input
        ),
    }
}

fn block<F>(input: &str, mut field_parser: F) -> Result<&str, String>
where
    F: FnMut(&str, String) -> Result<&str, String>,
{
    let mut input = skip(input, "{", "Expected an opening brace")?;

    loop {
        if let IResult::Done(rem, _) = punct!(input, "}") {
            break Ok(rem);
        }

        let (remaining, key) = match string_or_ident(input) {
            IResult::Done(rem, value) => (rem, value),
            IResult::Error => fail!("Expected an option name", input),
        };

        let remaining = field_parser(remaining, key)?;

        if let IResult::Done(rem, _) = punct!(remaining, "}") {
            break Ok(rem);
        }

        input = skip(remaining, ",", "Expected a comma or a closing brace")?;
    }
}

fn skip_colon(input: &str) -> Result<&str, String> {
    skip(input, ":", "Expected a colon")
}

fn skip<'a>(input: &'a str, symbol: &'static str, msg: &str) -> Result<&'a str, String> {
    match punct!(input, symbol) {
        IResult::Done(rem, _) => Ok(rem),
        IResult::Error => fail!(msg, input),
    }
}

#[cfg(test)]
mod macro_input_tests {
    use super::*;

    #[test]
    fn barebones_input() {
        assert_eq!(
            macro_input(r#" "Bob", "{}" "#),
            Ok(MacroInput {
                name: "Bob".to_string(),
                sample_source: "{}".to_string(),
                options: Options::default(),
            })
        );
    }

    #[test]
    fn barebones_input_with_empty_options() {
        assert_eq!(
            macro_input(r#" "Bob", "{}", {} "#),
            Ok(MacroInput {
                name: "Bob".to_string(),
                sample_source: "{}".to_string(),
                options: Options::default(),
            })
        );
    }

    #[test]
    fn barebones_input_with_options_as_string_literal() {
        assert_eq!(
            macro_input(r#" "Bob", "{}", "{}" "#),
            Ok(MacroInput {
                name: "Bob".to_string(),
                sample_source: "{}".to_string(),
                options: Options::default(),
            })
        );
    }
}

#[cfg(test)]
mod options_tests {
    use super::*;

    #[test]
    fn parses_derives() {
        let mut expected = Options::default();
        expected.derives = "Foo, Bar".into();

        assert_eq!(
            options(
                r#"{
                "derives": "Foo, Bar",
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn rejects_unknown_options() {
        let result = options(
            r#"{
            "foo_opt": {},
        }"#,
        );

        assert!(
            result.is_err(),
            "Parse result was not Err, but:\n{:?}",
            result
        );
        if let Err(message) = result {
            assert!(
                message.contains("foo_opt"),
                "Error message was:\n'{}'",
                message
            );
        }
    }

    #[test]
    fn parses_empty_pointer_block() {
        let expected = Options::default();

        assert_eq!(
            options(
                r#"{
                "/foo/bar": {},
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_map_hint() {
        let mut expected = Options::default();
        expected
            .hints
            .push(("/foo/bar".to_string(), Hint::default_map()));

        assert_eq!(
            options(
                r#"{
                "/foo/bar": {
                    use_type: "map"
                },
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_opaque_type_hint() {
        let mut expected = Options::default();
        expected
            .hints
            .push(("/foo/bar".to_string(), Hint::opaque_type("FooBar")));

        assert_eq!(
            options(
                r#"{
                "/foo/bar": {
                    use_type: "FooBar"
                },
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_type_name_hint() {
        let mut expected = Options::default();
        expected
            .hints
            .push(("/baz".to_string(), Hint::type_name("SomeName")));

        assert_eq!(
            options(
                r#"{
                "/baz": {
                    type_name: "SomeName"
                },
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_pointer_to_root() {
        let expected = Options::default();

        assert_eq!(
            options(
                r#"{
                "": {},
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn parses_keys_given_as_bare_identifiers() {
        let mut expected = Options::default();
        expected.derives = "Foo, Bar".into();

        assert_eq!(
            options(
                r#"{
                derives: "Foo, Bar",
            }"#
            ),
            Ok(expected)
        );
    }

    #[test]
    fn trailing_comma_is_optional() {
        let mut expected = Options::default();
        expected.derives = "Foo, Bar".into();

        assert_eq!(
            options(
                r#"{
                "derives": "Foo, Bar"
            }"#
            ),
            Ok(expected.clone())
        );

        assert_eq!(
            options(
                r#"{
                "derives": "Foo, Bar",
            }"#
            ),
            Ok(expected)
        );
    }
}

#[cfg(test)]
mod full_macro_tests {
    use super::*;

    #[test]
    fn full_macro_accepts_barebones_macro() {
        assert_eq!(
            full_macro(r#"json_typegen!("Bob", "{}");"#),
            Ok(MacroInput {
                name: "Bob".to_string(),
                sample_source: "{}".to_string(),
                options: Options::default(),
            })
        );
    }

    #[test]
    fn full_macro_accepts_barebones_macro_with_options() {
        assert_eq!(
            full_macro(r#"json_typegen!("Bob", "{}", {});"#),
            Ok(MacroInput {
                name: "Bob".to_string(),
                sample_source: "{}".to_string(),
                options: Options::default(),
            })
        );
    }
}

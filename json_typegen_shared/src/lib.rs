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

use std::fs::File;
use serde_json::{ Value };
use std::collections::{ HashSet };
use quote::{ Tokens, Ident, ToTokens };
use std::ascii::AsciiExt;
use linked_hash_map::LinkedHashMap;
use inflector::Inflector;
use regex::Regex;

mod util;
mod inference;
mod hints;

use util::*;
use inference::*;
use hints::*;

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

const RUST_KEYWORDS_ARR: &'static [&'static str] = &["abstract", "alignof", "as", "become", "box",
    "break", "const", "continue", "crate", "do", "else", "enum", "extern", "false", "final", "fn",
    "for", "if", "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "offsetof",
    "override", "priv", "proc", "pub", "pure", "ref", "return", "Self", "self", "sizeof", "static",
    "struct", "super", "trait", "true", "type", "typeof", "unsafe", "unsized", "use", "virtual",
    "where", "while", "yield"];

lazy_static! {
    static ref RUST_KEYWORDS: HashSet<&'static str> = {
        RUST_KEYWORDS_ARR.iter().cloned().collect()
    };
}

pub enum SampleSource<'a> {
    Url(&'a str),
    File(&'a str),
    Text(&'a str),
}

#[derive(Debug, PartialEq)]
pub enum MissingFields {
    Fail,
    UseDefault,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Visibility {
    Private,
    Pub,
    PubRestricted(String)
}

impl ToTokens for Visibility {
    fn to_tokens(&self, tokens: &mut Tokens) {
        use Visibility::*;
        match *self {
            Private => {},
            Pub => {
                tokens.append("pub");
            }
            PubRestricted(ref path) => {
                tokens.append("pub(");
                tokens.append(path);
                tokens.append(")");
            }
        }
    }
}

pub enum FieldVisibility {
    Inherited,
    Specified(Visibility)
}

pub struct Options {
    pub extern_crate: bool,
    pub runnable: bool,
    pub missing_fields: MissingFields,
    pub deny_unknown_fields: bool,
    pub allow_option_vec: bool,
    pub type_visibility: Visibility,
    pub field_visibility: FieldVisibility,
    pub derives: String,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            extern_crate: false,
            runnable: false,
            missing_fields: MissingFields::Fail,
            deny_unknown_fields: false,
            allow_option_vec: false,
            type_visibility: Visibility::Private,
            field_visibility: FieldVisibility::Inherited,
            derives: "Default, Debug, Clone, PartialEq, Serialize, Deserialize".into(),
        }
    }
}

struct Ctxt {
    options: Options,
    type_names: HashSet<String>,
}

macro_rules! some_if {
    ($cond:expr, $then:expr) => ({
        if $cond {
            Some($then)
        } else {
            None
        }
    })
}

pub fn from_str_with_defaults(name: &str, json: &str) -> Result<Tokens> {
    codegen(name, &SampleSource::Text(json), Options::default())
}

pub fn codegen(name: &str, source: &SampleSource, mut options: Options) -> Result<Tokens> {
    let sample = get_and_parse_sample(source)?;
    let name = handle_pub_in_name(name, &mut options);

    let hints = Hints::new();

    let mut ctxt = Ctxt {
        options: options,
        type_names: HashSet::new(),
    };
    let shape = value_to_shape(&sample, &hints);
    let (type_name, defs) = generate_type_from_shape(&mut ctxt, name, &shape);

    let example = some_if!(ctxt.options.runnable, {
        ctxt.options.extern_crate = true;
        usage_example(&type_name)
    });

    let crates = some_if!(ctxt.options.extern_crate, quote! {
        #[macro_use]
        extern crate serde_derive;
        extern crate serde_json;
    });

    Ok(quote! {
        #crates

        #defs

        #example
    })
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
                Some(restriction) => Visibility::PubRestricted(restriction.as_str().to_owned()),
                None => Visibility::Pub,
            };
            captures.name("name").unwrap().as_str()
        }
        None => {
            // If there is no visibility specified here, we want to use whatever is set elsewhere
            name
        }
    }
}

fn usage_example(type_id: &Tokens) -> Tokens {
    let var_id = Ident::from(snake_case(type_id.as_str()));

    quote! {
        fn main() {
            let #var_id: #type_id = Default::default();

            let serialized = serde_json::to_string(&#var_id).unwrap();

            println!("serialized = {}", serialized);

            let deserialized: #type_id = serde_json::from_str(&serialized).unwrap();

            println!("deserialized = {:?}", deserialized);
        }
    }
}

fn generate_type_from_shape(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Tokens, Option<Tokens>) {
    use Shape::*;
    match *shape {
        Null | Any | Bottom => (quote! { ::serde_json::Value }, None),
        Bool => (quote! { bool }, None),
        StringT => (quote! { String }, None),
        Integer => (quote! { i64 }, None),
        Floating => (quote! { f64 }, None),
        Tuple(ref shapes, _n) => {
            let folded = inference::fold_shapes(shapes.clone());
            if folded == Any && shapes.iter().any(|s| s != &Any) {
                generate_tuple_type(ctxt, path, shapes)
            } else {
                generate_vec_type(ctxt, path, &folded)
            }
        }
        VecT { elem_type: ref e } => {
            generate_vec_type(ctxt, path, e)
        }
        Struct { fields: ref map } => {
            generate_struct_from_field_shapes(ctxt, path, map)
        }
        MapT { val_type: ref v } => {
            generate_map_type(ctxt, path, v)
        }
        Optional(ref e) => {
            let (inner, defs) = generate_type_from_shape(ctxt, path, e);
            match ctxt.options.missing_fields {
                MissingFields::Fail => (quote! { Option<#inner> }, defs),
                MissingFields::UseDefault => (quote! { #inner }, defs),
            }
        }
    }
}

fn generate_vec_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Tokens, Option<Tokens>) {
    let singular = path.to_singular();
    let (inner, defs) = generate_type_from_shape(ctxt, &singular, shape);
    (quote! { Vec<#inner> }, defs)
}

fn generate_map_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Tokens, Option<Tokens>) {
    let singular = path.to_singular();
    let (inner, defs) = generate_type_from_shape(ctxt, &singular, shape);
    (quote! { ::std::collections::HashMap<String, #inner> }, defs)
}

fn generate_tuple_type(ctxt: &mut Ctxt, path: &str, shapes: &Vec<Shape>) -> (Tokens, Option<Tokens>) {
    let mut types = Vec::new();
    let mut defs = quote!{ };

    for shape in shapes {
        let (typ, def) = generate_type_from_shape(ctxt, path, shape);
        types.push(typ);
        if let Some(tokens) = def {
            defs.append(tokens);
        }
    }

    let typ = quote!{
        (#(#types),*)
    };
    (typ, Some(defs))
}

fn field_name(name: &str, used_names: &HashSet<String>) -> String {
    type_or_field_name(name, used_names, "field", snake_case)
}

fn type_name(name: &str, used_names: &HashSet<String>) -> String {
    type_or_field_name(name, used_names, "GeneratedType", type_case)
}

fn type_or_field_name(name: &str,
                      used_names: &HashSet<String>,
                      default_name: &str,
                      case_fn: fn(&str) -> String)
                      -> String {
    let name = name.trim();
    let mut output_name = if let Some(c) = name.chars().next() {
        if c.is_ascii() && c.is_numeric() {
            let temp = String::from("n") + name;
            case_fn(&temp)
        } else {
            case_fn(name)
        }
    } else {
        case_fn(name)
    };
    if RUST_KEYWORDS.contains::<str>(&output_name) {
        output_name.push_str("_field");
    }
    if output_name == "" {
        output_name.push_str(default_name);
    }
    if !used_names.contains(&output_name) {
        return output_name;
    }
    for n in 2.. {
        let temp = format!("{}{}", output_name, n);
        if !used_names.contains(&temp) {
            return temp;
        }
    }
    unreachable!()
}

fn collapse_option_vec<'a>(ctxt: &mut Ctxt,
                           typ: &'a Shape)
                           -> (Option<Tokens>, &'a Shape) {
    if !ctxt.options.allow_option_vec && ctxt.options.missing_fields != MissingFields::UseDefault {
        if let Shape::Optional(ref inner) = *typ {
            if let Shape::VecT { .. } = **inner {
                return (Some(quote! { #[serde(default)] }), &**inner);
            }
        }
    }
    (None, typ)
}

fn generate_struct_from_field_shapes(
        ctxt: &mut Ctxt,
        path: &str,
        map: &LinkedHashMap<String, Shape>) -> (Tokens, Option<Tokens>) {
    let type_name = type_name(path, &ctxt.type_names);
    ctxt.type_names.insert(type_name.clone());
    let ident = Ident::from(type_name);
    let visibility = ctxt.options.type_visibility.clone();
    let field_visibility = match ctxt.options.field_visibility {
        FieldVisibility::Inherited => visibility.clone(),
        FieldVisibility::Specified(ref v) => v.clone(),
    };

    let mut field_names = HashSet::new();
    let mut defs = Vec::new();

    let fields: Vec<Tokens> = map.iter()
        .map(|(name, typ)| {
            let field_name = field_name(name, &field_names);
            field_names.insert(field_name.clone());
            let rename = some_if!(&field_name != name,
                quote! { #[serde(rename = #name)] });
            let field_ident = Ident::from(field_name);
            let (default, collapsed) = collapse_option_vec(ctxt, typ);
            let (field_type, child_defs) = generate_type_from_shape(ctxt, name, collapsed);
            defs.push(child_defs);
            quote! {
                #rename
                #default
                #field_visibility #field_ident: #field_type
            }
        })
        .collect();

    let derive_list = Ident::from(&*ctxt.options.derives);

    let unknown_fields = some_if!(ctxt.options.deny_unknown_fields,
        quote! { #[serde(deny_unknown_fields)] });

    let use_defaults = some_if!(ctxt.options.missing_fields == MissingFields::UseDefault,
        quote! { #[serde(default)] });

    let code = quote! {
        #[derive(#derive_list)]
        #unknown_fields
        #use_defaults
        #visibility struct #ident {
            #(#fields),*
        }

        #(#defs)*
    };

    (quote! { #ident }, Some(code))
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
        assert_eq!(options.type_visibility, Visibility::Pub);
        let name = handle_pub_in_name("pub(crate) Foo Bar", &mut options);
        assert_eq!(name, "Foo Bar");
        assert_eq!(options.type_visibility, Visibility::PubRestricted("crate".to_string()));
        let name = handle_pub_in_name("pub(some::path) Foo", &mut options);
        assert_eq!(name, "Foo");
        assert_eq!(options.type_visibility, Visibility::PubRestricted("some::path".to_string()));
    }
}

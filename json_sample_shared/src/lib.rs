#[macro_use]
extern crate quote;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;

use std::fs::File;
use serde_json::{ Value, Map };
use std::collections::HashSet;
use quote::{ Tokens, Ident };

mod util;

use util::*;

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

pub struct Options {
    use_serde: bool,
    extern_crate: bool,
    runnable_example: bool
}

impl Default for Options {
    fn default() -> Options {
        Options {
            use_serde: true,
            extern_crate: false,
            runnable_example: false
        }
    }
}

struct Ctxt {
    options: Options
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

pub fn codegen_from_sample(name: &str, source: &SampleSource) -> Result<Tokens> {
    let sample = get_and_parse_sample(source)?;
    let mut ctxt = Ctxt {
        options: Options::default()
    };
    let (type_name, type_def) = generate_type_from_value(&mut ctxt, name, &sample);

    let example = some_if!(ctxt.options.runnable_example, {
        ctxt.options.extern_crate = true;
        usage_example(&type_name)
    });

    let crates = some_if!(ctxt.options.extern_crate, quote! {
        #[macro_use]
        extern crate serde_derive;
        extern crate serde_json;
    });

    if type_def.is_none() && !ctxt.options.runnable_example {
        return Err(ErrorKind::ExistingType(String::from(type_name.as_str())).into());
    }

    Ok(quote! {
        #crates

        #type_def

        #example
    })
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

fn generate_type_from_value(ctxt: &mut Ctxt, path: &str, value: &Value) -> (Tokens, Option<Tokens>) {
    match *value {
        Value::Null => (quote! { Option<::serde_json::Value> }, None),
        Value::Bool(_) => (quote! { bool }, None),
        Value::Number(ref n) => {
            if n.is_i64() {
                (quote! { i64 }, None)
            } else {
                (quote! { f64 }, None)
            }
        },
        Value::String(_) => (quote! { String }, None),
        Value::Array(ref values) => {
            generate_type_for_array(ctxt, path, values)
        },
        Value::Object(ref map) => {
            generate_struct_from_object(ctxt, path, map)
        }
    }
}

fn generate_type_for_array(ctxt: &mut Ctxt, path: &str, values: &[Value]) -> (Tokens, Option<Tokens>) {
    let mut defs = Vec::new();
    let mut types = HashSet::new();

    for value in values.iter() {
        let (elemtype, elemtype_def) = generate_type_from_value(ctxt, path, value);
        types.insert(elemtype.into_string());
        if let Some(def) = elemtype_def {
            defs.push(def);
        }
    }

    if types.len() == 1 {
        let ident = Ident::new(types.into_iter().next().unwrap());
        (quote! { Vec<#ident> }, defs.into_iter().next())
    } else {
        (quote! { Vec<::serde_json::Value> }, None)
    }
}

fn generate_struct_from_object(ctxt: &mut Ctxt, path: &str, map: &Map<String, Value>) -> (Tokens, Option<Tokens>) {
    let type_name = type_case(path);
    let ident = Ident::from(type_name);
    let mut defs = Vec::new();

    let fields: Vec<Tokens> = map.iter()
        .map(|(name, value)| {
            let mut field_name = snake_case(name);
            if RUST_KEYWORDS.contains::<str>(&field_name) {
                field_name.push_str("_field")
            }
            let rename = some_if!(&field_name != name,
                quote! { #[serde(rename = #name)] });

            let field_ident = Ident::from(field_name);
            let (fieldtype, fieldtype_def) = generate_type_from_value(ctxt, name, value);
            if let Some(def) = fieldtype_def {
                defs.push(def);
            }
            quote! {
                #rename
                #field_ident: #fieldtype
            }
        })
        .collect();

    let derives = if ctxt.options.use_serde {
        quote! { #[derive(Default, Debug, Clone, Serialize, Deserialize)] }
    } else {
        quote! { #[derive(Default, Debug, Clone)] }
    };

    let code = quote! {
        #derives
        struct #ident {
            #(#fields),*
        }

        #(#defs)*
    };

    (quote! { #ident }, Some(code))
}

fn get_and_parse_sample(source: &SampleSource) -> Result<Value> {
    let parse_result = match *source {
        SampleSource::Url(url) => serde_json::de::from_reader(reqwest::get(url)?),
        SampleSource::File(path) => serde_json::de::from_reader(File::open(path)?),
        SampleSource::Text(text) => serde_json::from_str(text),
    };
    Ok(parse_result.chain_err(|| "Unable to parse JSON sample")?)
}

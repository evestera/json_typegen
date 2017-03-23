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
use std::collections::{ HashSet, HashMap };
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
    pub use_serde: bool,
    pub extern_crate: bool,
    pub runnable: bool
}

impl Default for Options {
    fn default() -> Options {
        Options {
            use_serde: true,
            extern_crate: false,
            runnable: false
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

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
enum InferredType {
    Null,
    Any,
    Bool,
    StringT,
    Integer,
    Floating,
    VecT { elem_type: Box<InferredType> },
    Struct { fields: HashMap<String, InferredType> },
    Optional(Box<InferredType>)
}

#[allow(dead_code)]
fn unify(a: InferredType, b: InferredType) -> InferredType {
    if a == b {
        return a;
    }
    use InferredType::*;
    match (a, b) {
        (Integer, Floating) |
        (Floating, Integer) => Floating,
        (a, Null) | (Null, a) => make_optional(a),
        (a, Optional(b)) | (Optional(b), a) => make_optional(unify(a, *b)),
        (VecT { elem_type: e1 }, VecT { elem_type: e2 }) => {
            VecT { elem_type: Box::new(unify(*e1, *e2)) }
        }
        (Struct { fields: f1 }, Struct { fields: f2 }) => {
            Struct { fields: unify_struct_fields(f1, f2) }
        }
        _ => Any,
    }
}

fn make_optional(a: InferredType) -> InferredType {
    use InferredType::*;
    match a {
        Null | Any | Optional(_) => a,
        non_nullable => Optional(Box::new(non_nullable)),
    }
}

fn unify_struct_fields(mut f1: HashMap<String, InferredType>,
                       mut f2: HashMap<String, InferredType>)
                       -> HashMap<String, InferredType> {
    if f1 == f2 {
        return f1;
    }
    let mut unified = HashMap::new();
    for (key, val) in f1.drain() {
        match f2.remove(&key) {
            Some(val2) => {
                unified.insert(key, unify(val, val2));
            },
            None => {
                unified.insert(key, make_optional(val));
            }
        }
    }
    for (key, val) in f2.drain() {
        unified.insert(key, make_optional(val));
    }
    unified
}

pub fn from_str_with_defaults(name: &str, json: &str) -> Result<Tokens> {
    codegen(name, &SampleSource::Text(json), Options::default())
}

pub fn codegen(name: &str, source: &SampleSource, options: Options) -> Result<Tokens> {
    let sample = get_and_parse_sample(source)?;
    let mut ctxt = Ctxt {
        options: options
    };
    let (type_name, type_def) = generate_type_from_value(&mut ctxt, name, &sample);

    let example = some_if!(ctxt.options.runnable, {
        ctxt.options.extern_crate = true;
        usage_example(&type_name)
    });

    let crates = some_if!(ctxt.options.extern_crate, quote! {
        #[macro_use]
        extern crate serde_derive;
        extern crate serde_json;
    });

    if type_def.is_none() && !ctxt.options.runnable {
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

#[test]
fn test_unify() {
    use InferredType::*;
    assert_eq!(unify(Bool, Bool), Bool);
    assert_eq!(unify(Bool, Integer), Any);
    assert_eq!(unify(Integer, Floating), Floating);
    assert_eq!(unify(Null, Any), Any);
    assert_eq!(unify(Null, Bool), Optional(Box::new(Bool)));
    assert_eq!(unify(Null, Optional(Box::new(Integer))), Optional(Box::new(Integer)));
    assert_eq!(unify(Any, Optional(Box::new(Integer))), Any);
    assert_eq!(unify(Any, Optional(Box::new(Integer))), Any);
    assert_eq!(unify(Optional(Box::new(Integer)), Optional(Box::new(Floating))),
               Optional(Box::new(Floating)));
    assert_eq!(unify(Optional(Box::new(StringT)), Optional(Box::new(Integer))), Any);
}

// based on hashmap! macro from maplit crate
macro_rules! string_hashmap {
    ($($key:expr => $value:expr,)+) => { string_hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let mut _map = ::std::collections::HashMap::new();
            $(
                _map.insert($key.to_string(), $value);
            )*
            _map
        }
    };
}

#[test]
fn test_unify_struct_fields() {
    use InferredType::*;
    {
        let f1 = string_hashmap!{
            "a" => Integer,
            "b" => Bool,
            "c" => Integer,
            "d" => StringT,
        };
        let f2 = string_hashmap!{
            "a" => Integer,
            "c" => Floating,
            "d" => Null,
            "e" => Any,
        };
        assert_eq!(unify_struct_fields(f1, f2), string_hashmap!{
            "a" => Integer,
            "b" => Optional(Box::new(Bool)),
            "c" => Floating,
            "d" => Optional(Box::new(StringT)),
            "e" => Any,
        });
    }
}

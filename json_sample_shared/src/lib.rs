#[macro_use]
extern crate quote;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::collections::{ HashMap, HashSet };

pub mod spec;
mod util;

use spec::*;
use util::*;

pub enum SpecSource<'a> {
    Url(&'a str),
    File(&'a str),
    Text(&'a str),
}

#[derive(Debug)]
pub enum SpecError {
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),
    IoError(std::io::Error),
    MissingSource,
}

impl From<reqwest::Error> for SpecError {
    fn from(err: reqwest::Error) -> Self {
        SpecError::ReqwestError(err)
    }
}

impl From<serde_json::Error> for SpecError {
    fn from(err: serde_json::Error) -> Self {
        SpecError::JsonError(err)
    }
}

impl From<std::io::Error> for SpecError {
    fn from(err: std::io::Error) -> Self {
        SpecError::IoError(err)
    }
}

pub fn codegen_from_spec(name: &str, source: SpecSource) -> Result<quote::Tokens, SpecError> {
    let spec = get_spec(source)?;
    let name_id = quote::Ident::new(name);

    let paths = spec.paths;
    let mut ctxt = Ctxt::new();
    if let Some(definitions) = spec.definitions {
        process_refmap(&definitions, "#/definitions", &mut ctxt);
    }

    let fns = generate_fns(vec!["foo", "bar", "baz"]);

    let tokens = quote! {
        impl #name_id {
            #(#fns)*
        }
    };
    Ok(tokens)
}

enum TypeDef {
    Struct { name: String, fields: Vec<Field> },
    Enum { name: String, variants: Vec<String> },
    Alias { name: String, typ: String },
}

struct Field {
    name: String,
    original_name: Option<String>,
    typ: String
}

struct Ctxt {
    /// Map from spec path to type definitions
    types: HashMap<String, TypeDef>,
    /// Set of type names that are taken (same names as `types[x].name`)
    names: HashSet<String>,
    /// Should generated types with fields not marked as required be wrapped with Option?
    wrap_optional: bool
}

impl Ctxt {
    fn new() -> Self {
        Ctxt {
            types: HashMap::new(),
            names: HashSet::new(),
            wrap_optional: true
        }
    }
}

fn process_refmap(map: &HashMap<String, RefOr<SchemaObject>>,
                  current_path: &str,
                  ctxt: &mut Ctxt) -> Vec<Field> {
    let mut temp_path: String = String::from(current_path) + "/";
    let mut fields = Vec::new();

    for (name, ref_or_schema) in map.iter() {
        temp_path += name;
        let field_type = process_ref_or_schema(ref_or_schema, &temp_path, ctxt);
        let field_name = snake_case(name);
        let original_name = if &field_name != name { Some(name.clone()) } else { None };
        fields.push(Field {
            name: field_name,
            original_name: original_name,
            typ: field_type
        });
        temp_path.truncate(current_path.len() + 1);
    }

    fields
}

fn process_ref_or_schema(ref_or_schema: &RefOr<SchemaObject>,
                         current_path: &str,
                         ctxt: &mut Ctxt) -> String {
    match *ref_or_schema {
        RefOr::Ref { ref path } => {
            if !path.starts_with("#/definitions/") {
                panic!("Unsupported $ref target: {} {}", current_path, path);
            }
            // let def = TypeDef::Alias { name: type_name, typ: target_name };
            // ctxt.names.insert(def.name.clone());
            // ctxt.types.insert(current_path, def);
            typename_from_path(&path)
        },
        RefOr::Other(ref schema) => {
            process_schema(&schema, &current_path, ctxt)
        }
    }
}

fn process_schema(schema: &SchemaObject,
                  current_path: &str,
                  ctxt: &mut Ctxt) -> String {
    if let Some(ref typ) = schema.typ {
        if typ == "string" {
            return String::from("String");
        }
    }
    String::from("Value")
}

fn typename_from_path(path: &str) -> String {
    if path.starts_with("#/definitions/") {
        camel_case(&path[14..])
    } else {
        camel_case(&path)
    }
}

fn generate_fns(names: Vec<&str>) -> Vec<quote::Tokens> {
    names.iter()
        .map(|name| {
            let ident = quote::Ident::new(*name);
            quote! {
                fn #ident() {
                    println!("hello {}", #name);
                }
            }
        })
        .collect()
}

fn get_spec(source: SpecSource) -> Result<Spec, SpecError> {
    let parse_result = match source {
        SpecSource::Url(url) => serde_json::de::from_reader(reqwest::get(url)?),
        SpecSource::File(path) => serde_json::de::from_reader(File::open(path)?),
        SpecSource::Text(text) => serde_json::from_str(text),
    };
    parse_result.map_err(SpecError::JsonError)
}

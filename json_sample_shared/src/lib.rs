#[macro_use]
extern crate quote;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::fs::File;
use serde_json::{ Value, Map };
use std::collections::HashSet;

mod util;

use util::*;

pub enum SampleSource<'a> {
    Url(&'a str),
    File(&'a str),
    Text(&'a str),
}

#[derive(Debug)]
pub enum CodeGenError {
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),
    IoError(std::io::Error),
    MissingSource,
    ExistingType(String)
}

impl From<reqwest::Error> for CodeGenError {
    fn from(err: reqwest::Error) -> Self {
        CodeGenError::ReqwestError(err)
    }
}

impl From<serde_json::Error> for CodeGenError {
    fn from(err: serde_json::Error) -> Self {
        CodeGenError::JsonError(err)
    }
}

impl From<std::io::Error> for CodeGenError {
    fn from(err: std::io::Error) -> Self {
        CodeGenError::IoError(err)
    }
}

pub fn codegen_from_sample(name: &str, source: SampleSource) -> Result<quote::Tokens, CodeGenError> {
    let value = get_sample(source)?;
    let (type_name, type_def) = generate_type_from_value(name, &value);

    match type_def {
        Some(tokens) => Ok(tokens),
        None => Err(CodeGenError::ExistingType(String::from(type_name.as_str())))
    }
}

fn generate_type_from_value(path: &str, value: &Value) -> (quote::Tokens, Option<quote::Tokens>) {
    match *value {
        Value::Null => (quote!{ Option<::serde_json::Value> }, None),
        Value::Bool(_) => (quote!{ bool }, None),
        Value::Number(ref n) => {
            if n.is_i64() {
                (quote!{ i64 }, None)
            } else {
                (quote!{ f64 }, None)
            }
        },
        Value::String(_) => (quote!{ String }, None),
        Value::Array(ref values) => {
            generate_type_for_array(path, values)
        },
        Value::Object(ref map) => {
            generate_struct_from_object(path, map)
        }
    }
}

fn generate_type_for_array(path: &str, values: &Vec<Value>) -> (quote::Tokens, Option<quote::Tokens>) {
    let mut defs = Vec::new();
    let mut types = HashSet::new();

    for value in values.iter() {
        let (elemtype, elemtype_def) = generate_type_from_value(path, value);
        types.insert(elemtype.into_string());
        if let Some(def) = elemtype_def {
            defs.push(def);
        }
    }

    if types.len() == 1 {
        let ident = quote::Ident::new(types.into_iter().next().unwrap());
        (quote! { Vec<#ident> }, defs.into_iter().next())
    } else {
        (quote! { Vec<::serde_json::Value> }, None)
    }
}

fn generate_struct_from_object(path: &str, map: &Map<String, Value>) -> (quote::Tokens, Option<quote::Tokens>) {
    let type_name = type_case(path);
    let ident = quote::Ident::new(&type_name as &str);
    let mut defs = Vec::new();

    let fields: Vec<quote::Tokens> = map.iter()
        .map(|(name, value)| {
            let field_name = snake_case(name);
            let field_ident = quote::Ident::new(&field_name as &str);
            let (fieldtype, fieldtype_def) = generate_type_from_value(name, value);
            if let Some(def) = fieldtype_def {
                defs.push(def);
            }
            quote! {
                #field_ident: #fieldtype
            }
        })
        .collect();

    let code = quote! {
        struct #ident {
            #(#fields),*
        }

        #(#defs)*
    };

    (quote! { #ident }, Some(code))
}

fn get_sample(source: SampleSource) -> Result<Value, CodeGenError> {
    let parse_result = match source {
        SampleSource::Url(url) => serde_json::de::from_reader(reqwest::get(url)?),
        SampleSource::File(path) => serde_json::de::from_reader(File::open(path)?),
        SampleSource::Text(text) => serde_json::from_str(text),
    };
    parse_result.map_err(CodeGenError::JsonError)
}

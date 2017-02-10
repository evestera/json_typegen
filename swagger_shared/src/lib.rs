#[macro_use]
extern crate quote;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::Read;

mod spec;

use spec::*;

pub enum SpecSource<'a> {
    Url(&'a str),
    File(&'a str),
}

#[derive(Debug)]
pub enum SpecError {
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),
    IoError(std::io::Error),
    MissingSource
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
    if let Some(definitions) = spec.definitions {
        println!("We have some definitions");
    }

    let fns = generate_fns(vec!["foo", "bar", "baz"]);

    let tokens = quote! {
        impl #name_id {
            #(#fns)*
        }
    };
    Ok(tokens)
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
    match source {
        SpecSource::Url(url) => read_spec(reqwest::get(url)?),
        SpecSource::File(path) => read_spec(File::open(path)?)
    }
}

fn read_spec<T: Read>(reader: T) -> Result<Spec, SpecError> {
    let spec: Spec = serde_json::de::from_reader(reader)?;
    Ok(spec)
}



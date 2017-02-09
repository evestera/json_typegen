extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use proc_macro::TokenStream;
use syn::{MetaItem, NestedMetaItem, Attribute, Lit};
use std::fs::File;
use std::io::Read;

mod spec;

use spec::*;

#[derive(Debug)]
enum SpecError {
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

#[proc_macro_derive(Swagger, attributes(swagger))]
pub fn derive_swagger(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_macro_input(&source).unwrap();

    let expanded = expand_swagger(&ast).unwrap();
    expanded.parse().unwrap()
}

fn expand_swagger(ast: &syn::MacroInput) -> Result<quote::Tokens, SpecError> {
    let name = &ast.ident;
    let spec_source = get_spec_source(&ast.attrs)?;
    let spec = get_spec(spec_source)?;

    let paths = spec.paths;
    if let Some(definitions) = spec.definitions {
        println!("We have some definitions");
    }

    let fns = generate_fns(vec!["foo", "bar", "baz"]);

    let tokens = quote! {
        impl #name {
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

enum SpecSource<'a> {
    Url(&'a str),
    File(&'a str),
}

fn get_spec_source(attrs: &Vec<Attribute>) -> Result<SpecSource, SpecError> {
    for items in attrs.iter().filter_map(get_swagger_meta_items) {
        for item in items {
            if let &NestedMetaItem::MetaItem(MetaItem::NameValue(ref name, ref value)) = item {
                if name == "url" {
                    if let &Lit::Str(ref str, ref _style) = value {
                        return Ok(SpecSource::Url(str));
                    }
                }
                if name == "file" {
                    if let &Lit::Str(ref str, ref _style) = value {
                        return Ok(SpecSource::File(str));
                    }
                }
            }
        }
    }
    Err(SpecError::MissingSource)
}

fn get_swagger_meta_items(attr: &Attribute) -> Option<&Vec<NestedMetaItem>> {
    match attr.value {
        MetaItem::List(ref name, ref items) if name == "swagger" => {
            Some(items)
        }
        _ => None
    }
}

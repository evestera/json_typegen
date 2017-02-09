extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use proc_macro::TokenStream;
use serde_json::value::{Value, Map};

#[derive(Deserialize)]
struct Spec {
    swagger: String,
    info: Info,
    host: Option<String>,
    #[serde(rename = "basePath")]
    base_path: Option<String>,
    paths: Map<String, Value>,
    definitions: Option<Map<String, Value>>,
    parameters: Option<Map<String, Value>>,
}

#[derive(Deserialize)]
struct Info {
    title: String,
    version: String,
    description: Option<String>,
}

#[derive(Debug)]
enum SpecError {
    ReqwestError(reqwest::Error),
    JsonError(serde_json::Error),
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

#[proc_macro_derive(Swagger, attributes(url))]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_macro_input(&source).unwrap();

    let expanded = expand_swagger(&ast);
    expanded.parse().unwrap()
}

fn expand_swagger(ast: &syn::MacroInput) -> quote::Tokens {
    let name = &ast.ident;
    let url = get_spec_url(&ast.attrs).unwrap();
    let spec = get_spec(url).unwrap();

    let paths = spec.paths;
    if let Some(definitions) = spec.definitions {
        println!("We have some definitions");
    }

    let fns = generate_fns(vec!["foo", "bar", "baz"]);

    quote! {
        impl #name {
            #(#fns)*
        }
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

fn get_spec(url: &str) -> Result<Spec, SpecError> {
    let res = reqwest::get(url)?;
    let spec: Spec = serde_json::de::from_reader(res)?;
    Ok(spec)
}

fn get_spec_url(attrs: &Vec<syn::Attribute>) -> Option<&str> {
    for attr in attrs {
        if let syn::MetaItem::NameValue(ref name, ref value) = attr.value {
            if name == "url" {
                if let &syn::Lit::Str(ref string, ref _style) = value {
                    return Some(string);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}

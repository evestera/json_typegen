#![feature(proc_macro, proc_macro_lib)]

extern crate proc_macro;
use proc_macro::TokenStream;

extern crate syn;
extern crate hyper;
extern crate serde_json;

use serde_json::value::{Value, Map};

#[macro_use]
extern crate quote;

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
    let spec = get_spec(url);

    let paths = match spec.get("paths").unwrap() {
        &Value::Object(ref map) => map,
        _ => panic!("Invalid paths object")
    };

    println!("{:?}", paths);

    quote! {
        impl #name {
            fn foo() {
                println!("Hello expanded");
            }
        }
    }
}

fn get_spec(url: &str) -> Map<String, Value> {
    let client = hyper::client::Client::new();
    let res = client.get(url).send().unwrap();
    let json: Value = serde_json::de::from_reader(res).unwrap();
    match json {
        Value::Object(map) => map,
        _ => panic!("Invalid spec")
    }
}

fn get_spec_url(attrs: &Vec<syn::Attribute>) -> Option<&str> {
    for attr in attrs {
        if let syn::MetaItem::NameValue(ref name, ref value) = attr.value {
            if name == "url" {
                if let &syn::Lit::Str(ref string, ref style) = value {
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
    fn it_works() {
    }
}

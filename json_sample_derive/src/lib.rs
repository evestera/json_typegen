extern crate proc_macro;
extern crate syn;
extern crate quote;
extern crate json_sample_shared;

use json_sample_shared::{codegen, Options, SampleSource, Result, ErrorKind};
use syn::{MetaItem, NestedMetaItem, Attribute, Lit};
use proc_macro::TokenStream;

#[proc_macro_derive(json_sample, attributes(json_sample))]
pub fn derive_json_sample(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_macro_input(&source).unwrap();

    let expanded = expand_json_sample(&ast).unwrap();
    expanded.parse().unwrap()
}

fn expand_json_sample(ast: &syn::MacroInput) -> Result<quote::Tokens> {
    let name = get_name(&ast.attrs)?;
    let sample_source = get_sample_source(&ast.attrs)?;
    codegen(name.as_ref(), &sample_source, Options::default())
}

fn get_sample_source(attrs: &Vec<Attribute>) -> Result<SampleSource> {
    for items in attrs.iter().filter_map(get_json_sample_meta_items) {
        for item in items {
            if let &NestedMetaItem::MetaItem(MetaItem::NameValue(ref name, ref value)) = item {
                if name == "url" {
                    if let &Lit::Str(ref str, ref _style) = value {
                        return Ok(SampleSource::Url(str));
                    }
                }
                if name == "file" {
                    if let &Lit::Str(ref str, ref _style) = value {
                        return Ok(SampleSource::File(str));
                    }
                }
                if name == "str" {
                    if let &Lit::Str(ref str, ref _style) = value {
                        return Ok(SampleSource::Text(str));
                    }
                }
            }
        }
    }
    Err(ErrorKind::MissingSource.into())
}

fn get_name(attrs: &Vec<Attribute>) -> Result<&str> {
    for items in attrs.iter().filter_map(get_json_sample_meta_items) {
        for item in items {
            if let &NestedMetaItem::MetaItem(MetaItem::NameValue(ref name, ref value)) = item {
                if name == "name" {
                    if let &Lit::Str(ref s, ref _style) = value {
                        return Ok(s);
                    }
                }
            }
        }
    }
    Ok("JsonRoot")
}

fn get_json_sample_meta_items(attr: &Attribute) -> Option<&Vec<NestedMetaItem>> {
    match attr.value {
        MetaItem::List(ref name, ref items) if name == "json_sample" => {
            Some(items)
        }
        _ => None
    }
}

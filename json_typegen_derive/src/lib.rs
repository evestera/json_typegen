extern crate proc_macro;
extern crate syn;
extern crate quote;
extern crate json_typegen_shared;

use json_typegen_shared::{codegen, Options, SampleSource, Result, ErrorKind, infer_source_type};
use syn::{MetaItem, NestedMetaItem, Attribute, Lit};
use proc_macro::TokenStream;

#[proc_macro_derive(json_types, attributes(json_typegen))]
pub fn derive_json_typegen(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let ast = syn::parse_macro_input(&source).unwrap();

    let expanded = expand_json_typegen(&ast).unwrap();
    expanded.parse().unwrap()
}

fn expand_json_typegen(ast: &syn::MacroInput) -> Result<quote::Tokens> {
    let name = get_name(&ast.attrs)?;
    let sample_source = get_sample_source(&ast.attrs)?;
    codegen(name.as_ref(), &sample_source, Options::default())
}

fn get_sample_source(attrs: &Vec<Attribute>) -> Result<SampleSource> {
    for items in attrs.iter().filter_map(get_json_typegen_meta_items) {
        for item in items {
            if let &NestedMetaItem::MetaItem(MetaItem::NameValue(ref name, ref value)) = item {
                if name == "url" {
                    if let &Lit::Str(ref s, ref _style) = value {
                        return Ok(SampleSource::Url(s));
                    }
                }
                if name == "file" {
                    if let &Lit::Str(ref s, ref _style) = value {
                        return Ok(SampleSource::File(s));
                    }
                }
                if name == "str" {
                    if let &Lit::Str(ref s, ref _style) = value {
                        return Ok(SampleSource::Text(s));
                    }
                }
                if name == "source" {
                    if let &Lit::Str(ref s, ref _style) = value {
                        return Ok(infer_source_type(s));
                    }
                }
            }
        }
    }
    Err(ErrorKind::MissingSource.into())
}

fn get_name(attrs: &Vec<Attribute>) -> Result<&str> {
    for items in attrs.iter().filter_map(get_json_typegen_meta_items) {
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

fn get_json_typegen_meta_items(attr: &Attribute) -> Option<&Vec<NestedMetaItem>> {
    match attr.value {
        MetaItem::List(ref name, ref items) if name == "json_typegen" => {
            Some(items)
        }
        _ => None
    }
}

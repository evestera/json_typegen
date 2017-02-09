extern crate proc_macro;
extern crate syn;
extern crate quote;
extern crate swagger_shared;

use swagger_shared::{codegen_from_spec, SpecSource, SpecError};
use syn::{MetaItem, NestedMetaItem, Attribute, Lit};
use proc_macro::TokenStream;

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
    codegen_from_spec(name.as_ref(), spec_source)
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
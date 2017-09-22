use std::collections::{ HashSet };
use quote::{ Tokens, Ident };
use std::ascii::AsciiExt;
use linked_hash_map::LinkedHashMap;
use inflector::Inflector;

use inference::{self, Shape};
use util::{snake_case, type_case};
use options::Options;

pub struct Ctxt {
    options: Options,
    type_names: HashSet<String>,
}

pub fn shape_to_example_program(name: &str, shape: &Shape, options: Options) -> Tokens {
    let (type_name, defs) = shape_to_type_defs(name, &shape, options);

    let var_name = Ident::from(snake_case(type_name.as_str()));

    quote! {
        #[macro_use]
        extern crate serde_derive;
        extern crate serde_json;

        #defs

        fn main() {
            let #var_name = #type_name::default();
            let serialized = serde_json::to_string(&#var_name).unwrap();
            println!("serialized = {}", serialized);
            let deserialized: #type_name = serde_json::from_str(&serialized).unwrap();
            println!("deserialized = {:?}", deserialized);
        }
    }
}

pub fn shape_to_type_defs(name: &str, shape: &Shape, options: Options) -> (Tokens, Option<Tokens>) {
    let mut ctxt = Ctxt {
        options: options,
        type_names: HashSet::new(),
    };

    type_from_shape(&mut ctxt, name, shape)
}

fn type_from_shape(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Tokens, Option<Tokens>) {
    use inference::Shape::*;
    match *shape {
        Null | Any | Bottom => (quote! { ::serde_json::Value }, None),
        Bool => (quote! { bool }, None),
        StringT => (quote! { String }, None),
        Integer => (quote! { i64 }, None),
        Floating => (quote! { f64 }, None),
        Tuple(ref shapes, _n) => {
            let folded = inference::fold_shapes(shapes.clone());
            if folded == Any && shapes.iter().any(|s| s != &Any) {
                generate_tuple_type(ctxt, path, shapes)
            } else {
                generate_vec_type(ctxt, path, &folded)
            }
        }
        VecT { elem_type: ref e } => {
            generate_vec_type(ctxt, path, e)
        }
        Struct { fields: ref map } => {
            generate_struct_from_field_shapes(ctxt, path, map)
        }
        MapT { val_type: ref v } => {
            generate_map_type(ctxt, path, v)
        }
        Opaque(ref t) => {
            let ident = Ident::from(t.clone());
            (quote!{ #ident }, None)
        }
        Optional(ref e) => {
            let (inner, defs) = type_from_shape(ctxt, path, e);
            if ctxt.options.use_default_for_missing_fields {
                (quote! { #inner }, defs)
            } else {
                (quote! { Option<#inner> }, defs)
            }
        }
    }
}

fn generate_vec_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Tokens, Option<Tokens>) {
    let singular = path.to_singular();
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (quote! { Vec<#inner> }, defs)
}

fn generate_map_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Tokens, Option<Tokens>) {
    let singular = path.to_singular();
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (quote! { ::std::collections::HashMap<String, #inner> }, defs)
}

fn generate_tuple_type(ctxt: &mut Ctxt, path: &str, shapes: &Vec<Shape>) -> (Tokens, Option<Tokens>) {
    let mut types = Vec::new();
    let mut defs = quote!{ };

    for shape in shapes {
        let (typ, def) = type_from_shape(ctxt, path, shape);
        types.push(typ);
        if let Some(tokens) = def {
            defs.append(tokens);
        }
    }

    let typ = quote!{
        (#(#types),*)
    };
    (typ, Some(defs))
}

fn field_name(name: &str, used_names: &HashSet<String>) -> String {
    type_or_field_name(name, used_names, "field", snake_case)
}

fn type_name(name: &str, used_names: &HashSet<String>) -> String {
    type_or_field_name(name, used_names, "GeneratedType", type_case)
}

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

fn type_or_field_name(name: &str,
                      used_names: &HashSet<String>,
                      default_name: &str,
                      case_fn: fn(&str) -> String)
                      -> String {
    let name = name.trim();
    let mut output_name = if let Some(c) = name.chars().next() {
        if c.is_ascii() && c.is_numeric() {
            let temp = String::from("n") + name;
            case_fn(&temp)
        } else {
            case_fn(name)
        }
    } else {
        case_fn(name)
    };
    if RUST_KEYWORDS.contains::<str>(&output_name) {
        output_name.push_str("_field");
    }
    if output_name == "" {
        output_name.push_str(default_name);
    }
    if !used_names.contains(&output_name) {
        return output_name;
    }
    for n in 2.. {
        let temp = format!("{}{}", output_name, n);
        if !used_names.contains(&temp) {
            return temp;
        }
    }
    unreachable!()
}

fn collapse_option_vec<'a>(ctxt: &mut Ctxt,
                           typ: &'a Shape)
                           -> (Option<Tokens>, &'a Shape) {
    if !(ctxt.options.allow_option_vec || ctxt.options.use_default_for_missing_fields) {
        if let Shape::Optional(ref inner) = *typ {
            if let Shape::VecT { .. } = **inner {
                return (Some(quote! { #[serde(default)] }), &**inner);
            }
        }
    }
    (None, typ)
}

fn generate_struct_from_field_shapes(
        ctxt: &mut Ctxt,
        path: &str,
        map: &LinkedHashMap<String, Shape>) -> (Tokens, Option<Tokens>) {
    let type_name = type_name(path, &ctxt.type_names);
    ctxt.type_names.insert(type_name.clone());
    let ident = Ident::from(type_name);
    let visibility = ctxt.options.type_visibility.clone();
    let field_visibility = match ctxt.options.field_visibility {
        None => visibility.clone(),
        Some(ref v) => v.clone(),
    };

    let mut field_names = HashSet::new();
    let mut defs = Vec::new();

    let fields: Vec<Tokens> = map.iter()
        .map(|(name, typ)| {
            let field_name = field_name(name, &field_names);
            field_names.insert(field_name.clone());
            let rename = some_if!(&field_name != name,
                quote! { #[serde(rename = #name)] });
            let field_ident = Ident::from(field_name);
            let field_visibility = Ident::from(field_visibility.clone());
            let (default, collapsed) = collapse_option_vec(ctxt, typ);
            let (field_type, child_defs) = type_from_shape(ctxt, name, collapsed);
            defs.push(child_defs);
            quote! {
                #rename
                #default
                #field_visibility #field_ident: #field_type
            }
        })
        .collect();

    let derive_list = Ident::from(&*ctxt.options.derives);

    let unknown_fields = some_if!(ctxt.options.deny_unknown_fields,
        quote! { #[serde(deny_unknown_fields)] });

    let use_defaults = some_if!(ctxt.options.use_default_for_missing_fields,
        quote! { #[serde(default)] });

    let visibility = Ident::from(visibility);

    let code = quote! {
        #[derive(#derive_list)]
        #unknown_fields
        #use_defaults
        #visibility struct #ident {
            #(#fields),*
        }

        #(#defs)*
    };

    (quote! { #ident }, Some(code))
}

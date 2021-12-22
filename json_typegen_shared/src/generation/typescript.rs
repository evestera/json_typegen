use linked_hash_map::LinkedHashMap;
use std::collections::HashSet;

use crate::options::Options;
use crate::shape::{self, Shape};
use crate::to_singular::to_singular;
use crate::util::type_case;

pub struct Ctxt {
    options: Options,
    type_names: HashSet<String>,
    created_interfaces: Vec<(Shape, Ident)>,
}

pub type Ident = String;
pub type Code = String;

pub fn typescript_types(name: &str, shape: &Shape, options: Options) -> Code {
    let mut ctxt = Ctxt {
        options,
        type_names: HashSet::new(),
        created_interfaces: Vec::new(),
    };

    if !matches!(shape, Shape::Struct { .. }) {
        // reserve the requested name
        ctxt.type_names.insert(name.to_string());
    }

    let (ident, code) = type_from_shape(&mut ctxt, name, shape);
    let mut code = code.unwrap_or_default();

    if ident != name {
        code = format!("export type {} = {};\n\n", name, ident) + &code;
    }

    code
}

fn type_from_shape(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    use crate::shape::Shape::*;
    match shape {
        Null | Any | Bottom => ("any".into(), None),
        Bool => ("boolean".into(), None),
        StringT => ("string".into(), None),
        Integer => ("number".into(), None),
        Floating => ("number".into(), None),
        Tuple(shapes, _n) => {
            let folded = shape::fold_shapes(shapes.clone());
            if folded == Any && shapes.iter().any(|s| s != &Any) {
                generate_tuple_type(ctxt, path, shapes)
            } else {
                generate_vec_type(ctxt, path, &folded)
            }
        }
        VecT { elem_type: e } => generate_vec_type(ctxt, path, e),
        Struct { fields } => generate_interface_type(ctxt, path, fields, shape),
        MapT { val_type: v } => generate_map_type(ctxt, path, v),
        Opaque(t) => (t.clone(), None),
        Optional(e) => {
            let (inner, defs) = type_from_shape(ctxt, path, e);
            if ctxt.options.use_default_for_missing_fields {
                (inner, defs)
            } else {
                (format!("{} | undefined", inner), defs)
            }
        }
    }
}

fn generate_vec_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = to_singular(path);
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (format!("{}[]", inner), defs)
}

fn generate_map_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = to_singular(path);
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (format!("{{ [key: string]: {} }}", inner), defs)
}

fn generate_tuple_type(ctxt: &mut Ctxt, path: &str, shapes: &[Shape]) -> (Ident, Option<Code>) {
    let mut types = Vec::new();
    let mut defs = Vec::new();

    for shape in shapes {
        let (typ, def) = type_from_shape(ctxt, path, shape);
        types.push(typ);
        if let Some(code) = def {
            defs.push(code)
        }
    }

    (format!("[{}]", types.join(", ")), Some(defs.join("\n\n")))
}

fn type_name(name: &str, used_names: &HashSet<String>) -> Ident {
    let name = name.trim();
    let mut output_name = if let Some(c) = name.chars().next() {
        if c.is_ascii_digit() {
            let temp = String::from("n") + name;
            type_case(&temp)
        } else {
            type_case(name)
        }
    } else {
        type_case(name)
    };
    if output_name.is_empty() {
        output_name.push_str("GeneratedType");
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

pub fn collapse_option(typ: &Shape) -> (bool, &Shape) {
    if let Shape::Optional(inner) = typ {
        return (true, &**inner);
    }
    (false, typ)
}

#[rustfmt::skip]
const RESERVED_WORDS: &[&str] = &["break", "case", "catch", "class", "const",
    "continue", "debugger", "default", "delete", "do", "else", "enum", "export", "extends", "false",
    "finally", "for", "function", "if", "import", "in", "instanceof", "new", "null", "return",
    "super", "switch", "this", "throw", "true", "try", "typeof", "var", "void", "while", "with",
    "implements", "interface", "let", "package", "private", "protected", "public", "static",
    "yield"];

pub fn is_ts_identifier(s: &str) -> bool {
    if RESERVED_WORDS.contains(&s) {
        return false;
    }

    if let Some((first, rest)) = s.as_bytes().split_first() {
        let first_valid = (b'a'..b'z').contains(first)
            || (b'A'..b'Z').contains(first)
            || *first == b'_'
            || *first == b'$';
        return first_valid
            && rest.iter().all(|b| {
                (b'a'..b'z').contains(b)
                    || (b'A'..b'Z').contains(b)
                    || *b == b'_'
                    || *b == b'$'
                    || (b'0'..b'9').contains(b)
            });
    }
    false
}

fn generate_interface_type(
    ctxt: &mut Ctxt,
    path: &str,
    field_shapes: &LinkedHashMap<String, Shape>,
    containing_shape: &Shape,
) -> (Ident, Option<Code>) {
    for (created_for_shape, ident) in ctxt.created_interfaces.iter() {
        if created_for_shape.is_acceptable_substitution_for(containing_shape) {
            return (ident.into(), None);
        }
    }

    let type_name = type_name(path, &ctxt.type_names);
    ctxt.type_names.insert(type_name.clone());
    ctxt.created_interfaces
        .push((containing_shape.clone(), type_name.clone()));

    let mut defs = Vec::new();

    let fields: Vec<Code> = field_shapes
        .iter()
        .map(|(name, typ)| {
            let (was_optional, collapsed) = collapse_option(typ);

            let (field_type, child_defs) = type_from_shape(ctxt, name, collapsed);

            if let Some(code) = child_defs {
                defs.push(code);
            }

            let escape_name = !is_ts_identifier(name);

            format!(
                "    {}{}{}{}: {};",
                if escape_name { "\"" } else { "" },
                name,
                if escape_name { "\"" } else { "" },
                if was_optional { "?" } else { "" },
                field_type
            )
        })
        .collect();

    let mut code = format!("export interface {} {{\n", type_name);

    if !fields.is_empty() {
        code += &fields.join("\n");
        code += "\n";
    }
    code += "}";

    if !defs.is_empty() {
        code += "\n\n";
        code += &defs.join("\n\n");
    }

    (type_name, Some(code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ts_identifier() {
        // Valid:
        assert!(is_ts_identifier("foobar"));
        assert!(is_ts_identifier("FOOBAR"));
        assert!(is_ts_identifier("foo_bar"));
        assert!(is_ts_identifier("$"));
        assert!(is_ts_identifier("foobar1"));

        // Invalid:
        assert!(!is_ts_identifier("1foobar"));
        assert!(!is_ts_identifier(""));
        assert!(!is_ts_identifier(" "));
        assert!(!is_ts_identifier(" foobar"));
        assert!(!is_ts_identifier("foobar "));
        assert!(!is_ts_identifier("foo bar"));
        assert!(!is_ts_identifier("foo.bar"));
        assert!(!is_ts_identifier("true"));
    }
}

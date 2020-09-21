use inflector::Inflector;
use lazy_static::lazy_static;
use linked_hash_map::LinkedHashMap;
use regex::Regex;
use std::collections::HashSet;

use crate::options::Options;
use crate::shape::{self, Shape};
use crate::util::{alias, type_case};

pub struct Ctxt {
    options: Options,
    type_names: HashSet<String>,
}

pub type Ident = String;
pub type Code = String;

pub fn typescript_types(name: &str, shape: &Shape, options: Options) -> (Ident, Option<Code>) {
    let mut ctxt = Ctxt {
        options,
        type_names: HashSet::new(),
    };

    let (ident, code) = type_from_shape(&mut ctxt, name, shape);
    alias(ident, name, code, &ctxt.options)
}

fn type_from_shape(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    use crate::shape::Shape::*;
    match *shape {
        Null | Any | Bottom => ("any".into(), None),
        Bool => ("boolean".into(), None),
        StringT => ("string".into(), None),
        Integer => ("number".into(), None),
        Floating => ("number".into(), None),
        Tuple(ref shapes, _n) => {
            let folded = shape::fold_shapes(shapes.clone());
            if folded == Any && shapes.iter().any(|s| s != &Any) {
                generate_tuple_type(ctxt, path, shapes)
            } else {
                generate_vec_type(ctxt, path, &folded)
            }
        }
        VecT { elem_type: ref e } => generate_vec_type(ctxt, path, e),
        Struct { fields: ref map } => generate_struct_from_field_shapes(ctxt, path, map),
        MapT { val_type: ref v } => generate_map_type(ctxt, path, v),
        Opaque(ref t) => (t.clone(), None),
        Optional(ref e) => {
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
    let singular = path.to_singular();
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (format!("{}[]", inner), defs)
}

fn generate_map_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = path.to_singular();
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
        if c.is_ascii() && c.is_numeric() {
            let temp = String::from("n") + name;
            type_case(&temp)
        } else {
            type_case(name)
        }
    } else {
        type_case(name)
    };
    if output_name == "" {
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

fn collapse_option(typ: &Shape) -> (bool, &Shape) {
    if let Shape::Optional(inner) = typ {
        return (true, &**inner);
    }
    (false, typ)
}

const RESERVED_WORDS_ARR: &[&str] = &[
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "new",
    "null",
    "return",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
];

lazy_static! {
    static ref RESERVED_WORDS: HashSet<&'static str> = RESERVED_WORDS_ARR.iter().cloned().collect();
}

fn is_ts_identifier(s: &str) -> bool {
    lazy_static! {
        static ref JS_IDENTIFIER_RE: Regex = Regex::new(r"^[a-zA-Z_$][a-zA-Z_$0-9]*$").unwrap();
    }
    JS_IDENTIFIER_RE.is_match(s) && !RESERVED_WORDS.contains(s)
}

fn generate_struct_from_field_shapes(
    ctxt: &mut Ctxt,
    path: &str,
    map: &LinkedHashMap<String, Shape>,
) -> (Ident, Option<Code>) {
    let type_name = type_name(path, &ctxt.type_names);
    ctxt.type_names.insert(type_name.clone());

    let mut defs = Vec::new();

    let fields: Vec<Code> = map
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

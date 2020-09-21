use inflector::Inflector;
use lazy_static::lazy_static;
use linked_hash_map::LinkedHashMap;
use std::collections::HashSet;

use crate::options::{Options, StringTransform};
use crate::shape::{self, Shape};
use crate::util::{alias, kebab_case, lower_camel_case, snake_case, type_case};

pub struct Ctxt {
    options: Options,
    type_names: HashSet<String>,
}

pub type Ident = String;
pub type Code = String;

pub fn kotlin_types(name: &str, shape: &Shape, options: Options) -> (Ident, Option<Code>) {
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
        Null | Any | Bottom => ("Any".into(), None),
        Bool => ("Boolean".into(), None),
        StringT => ("String".into(), None),
        Integer => ("Long".into(), None),
        Floating => ("Double".into(), None),
        Tuple(ref shapes, _n) => {
            let folded = shape::fold_shapes(shapes.clone());
            if shapes.len() <= 3 && folded == Any && shapes.iter().any(|s| s != &Any) {
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
                (format!("{}?", inner), defs)
            }
        }
    }
}

fn generate_vec_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = path.to_singular();
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (format!("List<{}>", inner), defs)
}

fn generate_map_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = path.to_singular();
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (format!("Map<String, {}>", inner), defs)
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

    (
        format!(
            "{}<{}>",
            match types.len() {
                2 => "Pair",
                3 => "Triple",
                _ => panic!("No n-tuple type exists for n of {}", types.len()),
            },
            types.join(", ")
        ),
        Some(defs.join("\n\n")),
    )
}

fn field_name(name: &str, used_names: &HashSet<String>) -> Ident {
    type_or_field_name(name, used_names, "field", lower_camel_case)
}

fn type_name(name: &str, used_names: &HashSet<String>) -> Ident {
    type_or_field_name(name, used_names, "GeneratedType", type_case)
}

// https://kotlinlang.org/docs/reference/keyword-reference.html
const KOTLIN_KEYWORDS_ARR: &[&str] = &[
    // Hard
    "as",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "val",
    "var",
    "when",
    "while",
    // Soft
    "by",
    "catch",
    "constructor",
    "delegate",
    "dynamic",
    "field",
    "file",
    "finally",
    "get",
    "import",
    "init",
    "param",
    "property",
    "receiver",
    "set",
    "setparam",
    "where",
    // Modifier
    "actual",
    "abstract",
    "annotation",
    "companion",
    "const",
    "crossinline",
    "data",
    "enum",
    "expect",
    "external",
    "final",
    "infix",
    "inline",
    "inner",
    "internal",
    "lateinit",
    "noinline",
    "open",
    "operator",
    "out",
    "override",
    "private",
    "protected",
    "public",
    "reified",
    "sealed",
    "suspend",
    "tailrec",
    "vararg",
    // Special
    "field",
    "it",
];

lazy_static! {
    static ref KOTLIN_KEYWORDS: HashSet<&'static str> =
        KOTLIN_KEYWORDS_ARR.iter().cloned().collect();
}

fn type_or_field_name(
    name: &str,
    used_names: &HashSet<String>,
    default_name: &str,
    case_fn: fn(&str) -> String,
) -> Ident {
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
    if KOTLIN_KEYWORDS.contains::<str>(&output_name) {
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

fn generate_struct_from_field_shapes(
    ctxt: &mut Ctxt,
    path: &str,
    map: &LinkedHashMap<String, Shape>,
) -> (Ident, Option<Code>) {
    let type_name = type_name(path, &ctxt.type_names);
    ctxt.type_names.insert(type_name.clone());

    let mut field_names = HashSet::new();
    let mut defs = Vec::new();

    let fields: Vec<Code> = map
        .iter()
        .map(|(name, typ)| {
            let field_name = field_name(name, &field_names);
            field_names.insert(field_name.clone());

            let needs_rename = if let Some(ref transform) = ctxt.options.property_name_format {
                &apply_transform(transform, &field_name) != name
            } else {
                &field_name != name
            };
            let mut field_code = String::new();
            if needs_rename {
                field_code += &format!("    @field:JsonProperty(\"{}\")\n", name)
            }

            let (field_type, child_defs) = type_from_shape(ctxt, name, typ);

            if let Some(code) = child_defs {
                defs.push(code);
            }

            format!("{}    val {}: {}", field_code, field_name, field_type)
        })
        .collect();

    let mut code = String::new();

    if let Some(ref transform) = ctxt.options.property_name_format {
        code += match transform {
            StringTransform::LowerCase => {
                "@JsonNaming(PropertyNamingStrategy.LowerCaseStrategy.class)\n"
            }
            StringTransform::PascalCase => {
                "@JsonNaming(PropertyNamingStrategy.UpperCamelCaseStrategy.class)\n"
            }
            StringTransform::SnakeCase => {
                "@JsonNaming(PropertyNamingStrategy.SnakeCaseStrategy.class)\n"
            }
            StringTransform::KebabCase => {
                "@JsonNaming(PropertyNamingStrategy.KebabCaseStrategy.class)\n"
            }
            _ => "",
        };
    }

    code += &format!("data class {}(\n", type_name);

    if !fields.is_empty() {
        code += &fields.join(",\n");
        code += "\n";
    }
    code += ")";

    if !defs.is_empty() {
        code += "\n\n";
        code += &defs.join("\n\n");
    }

    (type_name, Some(code))
}

fn apply_transform(transform: &StringTransform, field_name: &str) -> String {
    match transform {
        StringTransform::LowerCase => field_name.to_ascii_lowercase(),
        StringTransform::PascalCase => type_case(field_name),
        StringTransform::SnakeCase => snake_case(field_name),
        StringTransform::KebabCase => kebab_case(field_name),
        _ => field_name.to_string(),
    }
}

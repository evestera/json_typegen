use linked_hash_map::LinkedHashMap;
use std::collections::HashSet;
use unindent::unindent;

use crate::generation::serde_case::RenameRule;
use crate::options::{ImportStyle, Options, StringTransform};
use crate::shape::{self, Shape};
use crate::to_singular::to_singular;
use crate::util::{snake_case, type_case};

pub struct Ctxt {
    options: Options,
    type_names: HashSet<String>,
    imports: HashSet<String>,
    created_structs: Vec<(Shape, Ident)>,
}

pub type Ident = String;
pub type Code = String;

pub fn rust_program(name: &str, shape: &Shape, options: Options) -> Code {
    let defs = rust_types(name, shape, options);

    let var_name = snake_case(name);

    let main = unindent(&format!(
        r#"
        fn main() {{
            let {var_name} = {type_name}::default();
            let serialized = serde_json::to_string(&{var_name}).unwrap();
            println!("serialized = {{}}", serialized);
            let deserialized: {type_name} = serde_json::from_str(&serialized).unwrap();
            println!("deserialized = {{:?}}", deserialized);
        }}
        "#,
        var_name = var_name,
        type_name = name
    ));

    defs + "\n\n" + &main
}

pub fn rust_types(name: &str, shape: &Shape, options: Options) -> Code {
    let mut ctxt = Ctxt {
        options,
        type_names: HashSet::new(),
        imports: HashSet::new(),
        created_structs: Vec::new(),
    };

    if ctxt.options.import_style != ImportStyle::QualifiedPaths {
        ctxt.options.derives = ctxt
            .options
            .derives
            .clone()
            .split(',')
            .map(|s| import(&mut ctxt, s.trim()))
            .collect::<Vec<_>>()
            .join(", ");
    };

    if !matches!(shape, Shape::Struct { .. }) {
        // reserve the requested name
        ctxt.type_names.insert(name.to_string());
    }

    let (ident, code) = type_from_shape(&mut ctxt, name, shape);
    let mut code = code.unwrap_or_default();

    if ident != name {
        code = format!(
            "{} type {} = {};\n\n{}",
            ctxt.options.type_visibility, name, ident, code
        );
    }

    if !ctxt.imports.is_empty() {
        let mut imports: Vec<_> = ctxt.imports.drain().collect();
        imports.sort();
        let mut import_code = String::new();
        for import in imports {
            import_code += "use ";
            import_code += &import;
            import_code += ";\n";
        }
        import_code += "\n";
        code = import_code + &code;
    }

    code
}

fn type_from_shape(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    use crate::shape::Shape::*;
    match shape {
        Null | Any | Bottom => (import(ctxt, "serde_json::Value"), None),
        Bool => ("bool".into(), None),
        StringT => ("String".into(), None),
        Integer => ("i64".into(), None),
        Floating => ("f64".into(), None),
        Tuple(shapes, _n) => {
            let folded = shape::fold_shapes(shapes.clone());
            if folded == Any && shapes.iter().any(|s| s != &Any) {
                generate_tuple_type(ctxt, path, shapes)
            } else {
                generate_vec_type(ctxt, path, &folded)
            }
        }
        VecT { elem_type: e } => generate_vec_type(ctxt, path, e),
        Struct { fields } => generate_struct_type(ctxt, path, fields, shape),
        MapT { val_type: v } => generate_map_type(ctxt, path, v),
        Opaque(t) => (t.clone(), None),
        Optional(e) => {
            let (inner, defs) = type_from_shape(ctxt, path, e);
            if ctxt.options.use_default_for_missing_fields {
                (inner, defs)
            } else {
                (format!("Option<{}>", inner), defs)
            }
        }
    }
}

fn generate_vec_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = to_singular(path);
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (format!("Vec<{}>", inner), defs)
}

fn generate_map_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = to_singular(path);
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (
        format!(
            "{}<String, {}>",
            import(ctxt, "std::collections::HashMap"),
            inner
        ),
        defs,
    )
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

    (format!("({})", types.join(", ")), Some(defs.join("\n\n")))
}

fn field_name(name: &str, used_names: &HashSet<String>) -> Ident {
    type_or_field_name(name, used_names, "field", snake_case)
}

fn type_name(name: &str, used_names: &HashSet<String>) -> Ident {
    type_or_field_name(name, used_names, "GeneratedType", type_case)
}

const RUST_KEYWORDS: &[&str] = &[
    "abstract", "alignof", "as", "become", "box", "break", "const", "continue", "crate", "do",
    "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in", "let", "loop",
    "macro", "match", "mod", "move", "mut", "offsetof", "override", "priv", "proc", "pub", "pure",
    "ref", "return", "Self", "self", "sizeof", "static", "struct", "super", "trait", "true",
    "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield", "async",
    "await", "try",
];

fn type_or_field_name(
    name: &str,
    used_names: &HashSet<String>,
    default_name: &str,
    case_fn: fn(&str) -> String,
) -> Ident {
    let name = name.trim();
    let mut output_name = case_fn(name);
    if RUST_KEYWORDS.contains(&&*output_name) {
        output_name.push_str("_field");
    }
    if output_name.is_empty() {
        output_name.push_str(default_name);
    }
    if let Some(c) = output_name.chars().next() {
        if c.is_ascii_digit() {
            output_name = String::from("n") + &output_name;
        }
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

fn collapse_option_vec<'a>(ctxt: &mut Ctxt, typ: &'a Shape) -> (bool, &'a Shape) {
    if !(ctxt.options.allow_option_vec || ctxt.options.use_default_for_missing_fields) {
        if let Shape::Optional(inner) = typ {
            if let Shape::VecT { .. } = **inner {
                return (true, &**inner);
            }
        }
    }
    (false, typ)
}

fn import(ctxt: &mut Ctxt, qualified: &str) -> String {
    if !qualified.contains("::") {
        return qualified.into();
    }
    match ctxt.options.import_style {
        ImportStyle::AddImports => {
            ctxt.imports.insert(qualified.into());
            qualified.rsplit("::").next().unwrap().into()
        }
        ImportStyle::AssumeExisting => qualified.rsplit("::").next().unwrap().into(),
        ImportStyle::QualifiedPaths => qualified.into(),
    }
}

fn generate_struct_type(
    ctxt: &mut Ctxt,
    path: &str,
    field_shapes: &LinkedHashMap<String, Shape>,
    containing_shape: &Shape,
) -> (Ident, Option<Code>) {
    for (created_for_shape, ident) in ctxt.created_structs.iter() {
        if created_for_shape.is_acceptable_substitution_for(containing_shape) {
            return (ident.into(), None);
        }
    }

    let type_name = type_name(path, &ctxt.type_names);
    ctxt.type_names.insert(type_name.clone());
    ctxt.created_structs
        .push((containing_shape.clone(), type_name.clone()));

    let visibility = ctxt.options.type_visibility.clone();
    let field_visibility = match ctxt.options.field_visibility {
        None => visibility.clone(),
        Some(ref v) => v.clone(),
    };

    let mut field_names = HashSet::new();
    let mut defs = Vec::new();

    let fields: Vec<Code> = field_shapes
        .iter()
        .map(|(name, typ)| {
            let field_name = field_name(name, &field_names);
            field_names.insert(field_name.clone());

            let needs_rename = if let Some(ref transform) = ctxt.options.property_name_format {
                &to_rename_rule(transform).apply_to_field(&field_name) != name
            } else {
                &field_name != name
            };
            let mut field_code = String::new();
            if needs_rename {
                field_code += &format!("    #[serde(rename = \"{}\")]\n", name)
            }

            let (is_collapsed, collapsed) = collapse_option_vec(ctxt, typ);
            if is_collapsed {
                field_code += "    #[serde(default)]\n";
            }

            let (field_type, child_defs) = type_from_shape(ctxt, name, collapsed);

            if let Some(code) = child_defs {
                defs.push(code);
            }

            field_code += "    ";
            if !field_visibility.is_empty() {
                field_code += &field_visibility;
                field_code += " ";
            }

            format!("{}{}: {},", field_code, field_name, field_type)
        })
        .collect();

    let mut code = format!("#[derive({})]\n", ctxt.options.derives);

    if ctxt.options.deny_unknown_fields {
        code += "#[serde(deny_unknown_fields)]\n";
    }

    if ctxt.options.use_default_for_missing_fields {
        code += "#[serde(default)]\n";
    }

    if let Some(ref transform) = ctxt.options.property_name_format {
        if *transform != StringTransform::SnakeCase {
            code += &format!("#[serde(rename_all = \"{}\")]\n", serde_name(transform))
        }
    }

    if !visibility.is_empty() {
        code += &visibility;
        code += " ";
    }

    code += &format!("struct {} {{\n", type_name);

    if !fields.is_empty() {
        code += &fields.join("\n");
        code += "\n";
    }
    if ctxt.options.collect_additional {
        code += &format!(
            "    #[serde(flatten)]\n    additional_fields: {}<String, {}>,\n",
            import(ctxt, "std::collections::HashMap"),
            import(ctxt, "serde_json::Value"),
        )
    }
    code += "}";

    if !defs.is_empty() {
        code += "\n\n";
        code += &defs.join("\n\n");
    }

    (type_name, Some(code))
}

fn to_rename_rule(transform: &StringTransform) -> RenameRule {
    match transform {
        StringTransform::LowerCase => RenameRule::LowerCase,
        StringTransform::UpperCase => RenameRule::UPPERCASE,
        StringTransform::PascalCase => RenameRule::PascalCase,
        StringTransform::CamelCase => RenameRule::CamelCase,
        StringTransform::SnakeCase => RenameRule::SnakeCase,
        StringTransform::ScreamingSnakeCase => RenameRule::ScreamingSnakeCase,
        StringTransform::KebabCase => RenameRule::KebabCase,
        StringTransform::ScreamingKebabCase => RenameRule::ScreamingKebabCase,
    }
}

fn serde_name(transform: &StringTransform) -> &'static str {
    match transform {
        StringTransform::LowerCase => "lowercase",
        StringTransform::UpperCase => "UPPERCASE",
        StringTransform::PascalCase => "PascalCase",
        StringTransform::CamelCase => "camelCase",
        StringTransform::SnakeCase => "snake_case",
        StringTransform::ScreamingSnakeCase => "SCREAMING_SNAKE_CASE",
        StringTransform::KebabCase => "kebab-case",
        StringTransform::ScreamingKebabCase => "SCREAMING-KEBAB-CASE",
    }
}

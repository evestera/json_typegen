use inflector::Inflector;
use lazy_static::lazy_static;
use linked_hash_map::LinkedHashMap;
use std::collections::HashSet;

use crate::options::{ImportStyle, Options, StringTransform};
use crate::shape::{self, Shape};
use crate::util::{kebab_case, lower_camel_case, snake_case, type_case};
use crate::OutputMode;

struct Ctxt {
    options: Options,
    type_names: HashSet<String>,
    imports: HashSet<String>,
}

pub type Ident = String;
pub type Code = String;

pub fn kotlin_types(name: &str, shape: &Shape, options: Options) -> Code {
    let mut ctxt = Ctxt {
        options,
        type_names: HashSet::new(),
        imports: HashSet::new(),
    };

    if !matches!(shape, Shape::Struct { .. }) {
        // reserve the requested name
        ctxt.type_names.insert(name.to_string());
    }

    let (ident, code) = type_from_shape(&mut ctxt, name, shape);
    let mut code = code.unwrap_or(String::new());

    if ident != name {
        code = format!("typealias {} = {};\n\n", name, ident) + &code;
    }

    if !ctxt.imports.is_empty() {
        let mut imports: Vec<_> = ctxt.imports.drain().collect();
        imports.sort();
        let mut import_code = String::new();
        for import in imports {
            import_code += "import ";
            import_code += &import;
            import_code += "\n";
        }
        import_code += "\n";
        code = import_code + &code;
    }

    code
}

fn type_from_shape(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    use crate::shape::Shape::*;
    match *shape {
        Null | Any | Bottom => ("Any?".into(), None),
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
        Struct { fields: ref map } => generate_data_class(ctxt, path, map),
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
// Only hard keywords are restricted. Others don't cause any problems I know of.
#[rustfmt::skip]
const KOTLIN_KEYWORDS_ARR: &[&str] = &[
    "as", "break", "class", "continue", "do", "else", "false", "for", "fun", "if", "in",
    "interface", "is", "null", "object", "package", "return", "super", "this", "throw", "true",
    "try", "typealias", "val", "var", "when", "while",
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
    let mut output_name = case_fn(name);
    if KOTLIN_KEYWORDS.contains::<str>(&output_name) {
        output_name.push_str("_field");
    }
    if output_name == "" {
        output_name.push_str(default_name);
    }
    if let Some(c) = output_name.chars().next() {
        if c.is_ascii() && c.is_numeric() {
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

fn import(ctxt: &mut Ctxt, qualified: &str) -> String {
    match ctxt.options.import_style {
        ImportStyle::AddImports => {
            ctxt.imports.insert(qualified.into());
            qualified.rsplit('.').next().unwrap().into()
        }
        ImportStyle::AssumeExisting => qualified.rsplit('.').next().unwrap().into(),
        ImportStyle::QualifiedPaths => qualified.into(),
    }
}

fn generate_data_class(
    ctxt: &mut Ctxt,
    path: &str,
    map: &LinkedHashMap<String, Shape>,
) -> (Ident, Option<Code>) {
    if map.is_empty() {
        // Kotlin does not allow empty data classes, so use type for general unknown object
        // Once #30 is implemented: && !options.collect_unknown_properties
        return ("Map<String, Any>".into(), None);
    }

    let type_name = type_name(path, &ctxt.type_names);
    ctxt.type_names.insert(type_name.clone());

    let mut field_names = HashSet::new();
    let mut defs = Vec::new();

    let fields: Vec<Code> = map
        .iter()
        .map(|(name, typ)| {
            let field_name = field_name(name, &field_names);
            field_names.insert(field_name.clone());

            let mut field_code = String::new();
            if &apply_transform(ctxt, &field_name) != name {
                if ctxt.options.output_mode == OutputMode::KotlinJackson {
                    field_code += &format!(
                        "    @{}(\"{}\")\n",
                        import(ctxt, "com.fasterxml.jackson.annotation.JsonProperty"),
                        name
                    )
                } else if ctxt.options.output_mode == OutputMode::KotlinKotlinx {
                    field_code += &format!(
                        "    @{}(\"{}\")\n",
                        import(ctxt, "kotlinx.serialization.SerialName"),
                        name
                    )
                }
            }

            let (field_type, child_defs) = type_from_shape(ctxt, name, typ);

            if let Some(code) = child_defs {
                defs.push(code);
            }

            format!("{}    val {}: {}", field_code, field_name, field_type)
        })
        .collect();

    let mut code = String::new();

    code += &transform_annotation(ctxt);

    if ctxt.options.output_mode == OutputMode::KotlinKotlinx {
        code += &format!("@{}\n", import(ctxt, "kotlinx.serialization.Serializable"));
    }
    code += &format!("data class {}(\n", type_name);

    if !fields.is_empty() {
        code += &fields.join(",\n");
        code += ",\n";
    }
    if ctxt.options.collect_additional {
        code += &format!(
            "    @{}\n    @get:{}\n    val additionalFields: Map<String, Any> = mutableMapOf(),\n",
            import(ctxt, "com.fasterxml.jackson.annotation.JsonAnySetter"),
            import(ctxt, "com.fasterxml.jackson.annotation.JsonAnyGetter"),
        )
    }
    code += ")";

    if !defs.is_empty() {
        code += "\n\n";
        code += &defs.join("\n\n");
    }

    (type_name, Some(code))
}

fn apply_transform(ctxt: &Ctxt, field_name: &str) -> String {
    match (
        &ctxt.options.property_name_format,
        &ctxt.options.output_mode,
    ) {
        (Some(StringTransform::LowerCase), OutputMode::KotlinJackson) => {
            field_name.to_ascii_lowercase()
        }
        (Some(StringTransform::PascalCase), OutputMode::KotlinJackson) => type_case(field_name),
        (Some(StringTransform::SnakeCase), OutputMode::KotlinJackson) => snake_case(field_name),
        (Some(StringTransform::KebabCase), OutputMode::KotlinJackson) => kebab_case(field_name),
        _ => field_name.to_string(),
    }
}

fn jackson_naming_annotation(ctxt: &mut Ctxt, strategy: &str) -> String {
    format!(
        "@{}({}.{}::class)\n",
        import(ctxt, "com.fasterxml.jackson.databind.annotation.JsonNaming"),
        import(
            ctxt,
            "com.fasterxml.jackson.databind.PropertyNamingStrategies"
        ),
        strategy
    )
}

fn transform_annotation(ctxt: &mut Ctxt) -> String {
    match (
        &ctxt.options.property_name_format,
        &ctxt.options.output_mode,
    ) {
        (Some(StringTransform::LowerCase), OutputMode::KotlinJackson) => {
            jackson_naming_annotation(ctxt, "LowerCaseStrategy")
        }
        (Some(StringTransform::PascalCase), OutputMode::KotlinJackson) => {
            jackson_naming_annotation(ctxt, "UpperCamelCaseStrategy")
        }
        (Some(StringTransform::SnakeCase), OutputMode::KotlinJackson) => {
            jackson_naming_annotation(ctxt, "SnakeCaseStrategy")
        }
        (Some(StringTransform::KebabCase), OutputMode::KotlinJackson) => {
            jackson_naming_annotation(ctxt, "KebabCaseStrategy::class)")
        }
        _ => "".into(),
    }
}

#[cfg(test)]
mod kotlin_codegen_tests {
    use super::*;

    #[test]
    fn field_names_test() {
        fn field_name_test(from: &str, to: &str) {
            assert_eq!(
                field_name(from, &HashSet::new()),
                to.to_string(),
                r#"From "{}" to "{}""#,
                from,
                to
            );
        }

        field_name_test("valid", "valid");
        field_name_test("1", "n1");
        field_name_test("+1", "n1");
        field_name_test("", "field");
        field_name_test("object", "object_field");
    }
}

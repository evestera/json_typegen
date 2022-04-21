use linked_hash_map::LinkedHashMap;
use std::collections::HashSet;

use crate::options::{ImportStyle, Options, StringTransform};
use crate::shape::{self, Shape};
use crate::to_singular::to_singular;
use crate::util::{kebab_case, lower_camel_case, snake_case, type_case};

#[derive(PartialEq, PartialOrd, Ord, Eq, Hash, Clone, Copy)]
enum Import {
    Any,
    Optional,
    BaseModel,
    Field,
}

impl Import {
    fn pair(&self) -> (&'static str, &'static str) {
        match self {
            Import::Any => ("typing", "Any"),
            Import::Optional => ("typing", "Optional"),
            Import::BaseModel => ("pydantic", "BaseModel"),
            Import::Field => ("pydantic", "Field"),
        }
    }
    fn identifier(&self) -> &'static str {
        self.pair().1
    }
    fn qualified(&self) -> String {
        let (module, identifier) = self.pair();
        format!("{}.{}", module, identifier)
    }
}

struct Ctxt {
    options: Options,
    type_names: HashSet<String>,
    imports: HashSet<Import>,
    created_classes: Vec<(Shape, Ident)>,
}

pub type Ident = String;
pub type Code = String;

pub fn python_types(name: &str, shape: &Shape, options: Options) -> Code {
    let mut ctxt = Ctxt {
        options,
        type_names: HashSet::new(),
        imports: HashSet::new(),
        created_classes: Vec::new(),
    };

    if !matches!(shape, Shape::Struct { .. }) {
        // reserve the requested name
        ctxt.type_names.insert(name.to_string());
    }

    let (ident, code) = type_from_shape(&mut ctxt, name, shape);
    let mut code = code.unwrap_or_default();

    if !ctxt.imports.is_empty() {
        let mut imports: Vec<_> = ctxt.imports.drain().collect();
        imports.sort();
        let mut import_code = String::new();
        for import in imports {
            match ctxt.options.import_style {
                ImportStyle::AssumeExisting => {}
                ImportStyle::AddImports => {
                    let (module, identifier) = import.pair();
                    import_code += &format!("from {} import {}\n", module, identifier);
                }
                ImportStyle::QualifiedPaths => {
                    let (module, identifier) = import.pair();
                    import_code += &format!("import {}.{}\n", module, identifier);
                }
            }
        }
        import_code += "\n\n";
        code = import_code + &code;
    }

    if ident != name {
        code += &format!("\n\n{} = {}", name, ident);
    }
    code
}

fn type_from_shape(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    use crate::shape::Shape::*;
    match shape {
        Null | Any | Bottom => (import(ctxt, Import::Any), None),
        Bool => ("bool".into(), None),
        StringT => ("str".into(), None),
        Integer => ("int".into(), None),
        Floating => ("float".into(), None),
        Tuple(shapes, _n) => {
            let folded = shape::fold_shapes(shapes.clone());
            if shapes.len() <= 3 && folded == Any && shapes.iter().any(|s| s != &Any) {
                generate_tuple_type(ctxt, path, shapes)
            } else {
                generate_vec_type(ctxt, path, &folded)
            }
        }
        VecT { elem_type: e } => generate_vec_type(ctxt, path, e),
        Struct { fields } => generate_data_class(ctxt, path, fields, shape),
        MapT { val_type: v } => generate_map_type(ctxt, path, v),
        Opaque(t) => (t.clone(), None),
        Optional(e) => {
            let (inner, defs) = type_from_shape(ctxt, path, e);
            if ctxt.options.use_default_for_missing_fields {
                (inner, defs)
            } else {
                let optional = import(ctxt, Import::Optional);
                (format!("{}[{}]", optional, inner), defs)
            }
        }
    }
}

fn generate_vec_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = to_singular(path);
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (format!("list[{}]", inner), defs)
}

fn generate_map_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = to_singular(path);
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (format!("dict[str, {}]", inner), defs)
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
        format!("tuple[{}]", types.join(", ")),
        Some(defs.join("\n\n")),
    )
}

fn field_name(name: &str, used_names: &HashSet<String>) -> Ident {
    type_or_field_name(name, used_names, "field", snake_case)
}

fn type_name(name: &str, used_names: &HashSet<String>) -> Ident {
    type_or_field_name(name, used_names, "GeneratedType", type_case)
}

// https://docs.python.org/3/reference/lexical_analysis.html#keywords
#[rustfmt::skip]
const PYTHON_KEYWORDS: &[&str] = &[
    "False", "None", "True",
    "and", "as", "assert", "async", "await", "break", "class", "continue",
    "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
    "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass",
    "raise", "return", "try", "while", "with", "yield",
];

fn type_or_field_name(
    name: &str,
    used_names: &HashSet<String>,
    default_name: &str,
    case_fn: fn(&str) -> String,
) -> Ident {
    let name = name.trim();
    let mut output_name = case_fn(name);
    if PYTHON_KEYWORDS.contains(&&*output_name) {
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

fn import(ctxt: &mut Ctxt, import: Import) -> String {
    ctxt.imports.insert(import);
    match ctxt.options.import_style {
        ImportStyle::QualifiedPaths => import.qualified(),
        _ => import.identifier().into(),
    }
}

fn generate_data_class(
    ctxt: &mut Ctxt,
    path: &str,
    field_shapes: &LinkedHashMap<String, Shape>,
    containing_shape: &Shape,
) -> (Ident, Option<Code>) {
    for (created_for_shape, ident) in ctxt.created_classes.iter() {
        if created_for_shape.is_acceptable_substitution_for(containing_shape) {
            return (ident.into(), None);
        }
    }

    let type_name = type_name(path, &ctxt.type_names);
    ctxt.type_names.insert(type_name.clone());
    ctxt.created_classes
        .push((containing_shape.clone(), type_name.clone()));

    let mut field_names = HashSet::new();
    let mut defs = Vec::new();

    let fields: Vec<Code> = field_shapes
        .iter()
        .map(|(name, typ)| {
            let field_name = field_name(name, &field_names);
            field_names.insert(field_name.clone());

            let (field_type, child_defs) = type_from_shape(ctxt, name, typ);

            if let Some(code) = child_defs {
                defs.push(code);
            }

            let mut field_code = String::new();
            let transformed = &apply_transform(ctxt, &field_name);
            if transformed != name {
                field_code += &format!(" = {}(alias = \"{}\")", import(ctxt, Import::Field), transformed)
            }

            format!("    {}: {}{}", field_name, field_type, field_code)
        })
        .collect();

    let mut code = String::new();

    code += &format!(
        "class {}({}):\n",
        type_name,
        import(ctxt, Import::BaseModel)
    );

    if fields.is_empty() {
        code += "    pass\n";
    } else {
        code += &fields.join("\n");
        code += "\n";
    }

    if !defs.is_empty() {
        let mut d = defs.join("\n\n");
        d += "\n\n";
        d += &code;
        code = d;
    }

    (type_name, Some(code))
}

fn apply_transform(ctxt: &Ctxt, field_name: &str) -> String {
    match ctxt.options.property_name_format {
        Some(StringTransform::LowerCase) => field_name.to_ascii_lowercase(),
        Some(StringTransform::PascalCase) => type_case(field_name),
        Some(StringTransform::SnakeCase) => snake_case(field_name),
        Some(StringTransform::KebabCase) => kebab_case(field_name),
        Some(StringTransform::UpperCase) => field_name.to_ascii_uppercase(),
        Some(StringTransform::CamelCase) => lower_camel_case(field_name),
        Some(StringTransform::ScreamingSnakeCase) => snake_case(field_name).to_ascii_uppercase(),
        Some(StringTransform::ScreamingKebabCase) => kebab_case(field_name).to_ascii_uppercase(),
        None => field_name.to_string(),
    }
}

#[cfg(test)]
mod python_codegen_tests {
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
        field_name_test("def", "def_field");
    }
}

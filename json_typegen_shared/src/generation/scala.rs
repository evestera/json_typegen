use linked_hash_map::LinkedHashMap;
use std::collections::HashSet;

use crate::options::Options;
use crate::shape::{self, Shape};
use crate::to_singular::to_singular;
use crate::util::type_case;
use crate::ImportStyle;

struct Ctxt {
    options: Options,
    imports: HashSet<String>,
    type_names: HashSet<String>,
    created_case_classes: Vec<(Shape, TypeName)>,
}

type Ident = String;

#[derive(Clone)]
struct TypeName {
    pub raw: String,
    pub safe: Ident,
}

type Code = String;

pub fn scala_types(name: &str, shape: &Shape, options: Options) -> Code {
    let mut ctxt = Ctxt {
        options,
        imports: HashSet::new(),
        type_names: HashSet::new(),
        created_case_classes: Vec::new(),
    };
    let mut prelude = String::new();
    if ctxt.options.deny_unknown_fields {
        let config = import(&mut ctxt, "io.circe.generic.extras.Configuration");
        prelude += &format!("implicit val config: {config} = {config}.default",);
        prelude += ".withStrictDecoding";
        prelude += "\n\n";
    }

    let (_, type_code) = type_from_shape(&mut ctxt, name, shape);
    let body_code = type_code.unwrap_or_default();

    let mut imports = ctxt.imports.drain().collect::<Vec<String>>();
    imports.sort();
    let import_code = imports
        .iter()
        .fold(String::new(), |c, i| format!("{c}import {i}\n"));

    if import_code.is_empty() {
        prelude + &body_code
    } else {
        format!("{import_code}\n\n{prelude}{body_code}")
    }
}

fn type_from_shape(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    use crate::shape::Shape::*;
    match shape {
        Null => ("Option[Json]".into(), None),
        Any | Bottom => ("Json".into(), None),
        Bool => ("Boolean".into(), None),
        StringT => ("String".into(), None),
        Integer => ("Long".into(), None),
        Floating => ("Double".into(), None),
        Tuple(shapes, _n) => {
            let folded = shape::fold_shapes(shapes.clone());
            if folded == Any && shapes.iter().any(|s| s != &Any) {
                generate_tuple_type(ctxt, path, shapes)
            } else {
                generate_seq_type(ctxt, path, &folded)
            }
        }
        Optional(inner) => generate_option_type(ctxt, path, inner),
        Nullable(inner) => generate_option_type(ctxt, path, inner),
        VecT { elem_type } => generate_seq_type(ctxt, path, elem_type),
        MapT { val_type } => generate_map_type(ctxt, path, val_type),
        Struct { fields } => generate_case_class_type(ctxt, path, fields, shape),
        Opaque(name) => (name.clone(), None),
    }
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

fn generate_option_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = to_singular(path);
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (format!("Option[{}]", inner), defs)
}

fn generate_seq_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = to_singular(path);
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (format!("Seq[{}]", inner), defs)
}

fn generate_map_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> (Ident, Option<Code>) {
    let singular = to_singular(path);
    let (inner, defs) = type_from_shape(ctxt, &singular, shape);
    (format!("Map[String, {}]", inner), defs)
}

fn generate_case_class_type(
    ctxt: &mut Ctxt,
    path: &str,
    field_shapes: &LinkedHashMap<String, Shape>,
    containing_shape: &Shape,
) -> (Ident, Option<Code>) {
    let existing = ctxt.created_case_classes.iter().find_map(|(s, i)| {
        if s.is_acceptable_substitution_for(containing_shape) {
            Some(i.safe.clone())
        } else {
            None
        }
    });
    if let Some(ident) = existing {
        (ident, None)
    } else {
        let class_name = type_name(path, &ctxt.type_names);
        ctxt.type_names.insert(class_name.raw.clone());
        ctxt.created_case_classes
            .push((containing_shape.clone(), class_name.clone()));
        let mut defs: Vec<Code> = Vec::new();
        let mut fields: Vec<Code> = Vec::new();
        for (name, shape) in field_shapes.iter() {
            let field_name = field_name(name);
            let (field_type, child_defs) = type_from_shape(ctxt, name, shape);
            if let Some(code) = child_defs {
                defs.push(code);
            }
            let field = format!("{field_name}: {field_type}");
            fields.push(field)
        }
        let case_class = if !fields.is_empty() {
            format!(
                "case class {}(\n  {},\n)",
                class_name.safe,
                fields.join(",\n  ")
            )
        } else {
            format!("case class {}()", class_name.safe)
        };
        let mut code = case_class;
        code += "\n\n";
        code += &generate_codec(ctxt, &class_name);
        if !defs.is_empty() {
            code += "\n\n";
            code += &defs.join("\n\n");
        }
        (class_name.safe, Some(code))
    }
}

fn import(ctxt: &mut Ctxt, qualified: &str) -> Code {
    match qualified.rsplit(".").next() {
        None => qualified.into(),
        Some(value) => match ctxt.options.import_style {
            ImportStyle::AddImports => {
                ctxt.imports.insert(qualified.into());
                value.into()
            }
            ImportStyle::AssumeExisting => value.into(),
            ImportStyle::QualifiedPaths => qualified.into(),
        },
    }
}

fn generate_codec(ctxt: &mut Ctxt, name: &TypeName) -> Code {
    let is_configured = ctxt
        .imports
        .contains("io.circe.generic.extras.Configuration");
    let derive_codec = if is_configured {
        import(ctxt, "io.circe.generic.semiauto.deriveConfiguredCodec")
    } else {
        import(ctxt, "io.circe.generic.semiauto.deriveCodec")
    };
    let codec_type = import(ctxt, "io.circe.Codec");
    let codec_name = sanitize_name(&format!("codec{}", name.raw));
    format!(
        "implicit lazy val {}: {}[{}] = {}[{}]",
        codec_name, codec_type, name.safe, derive_codec, name.safe,
    )
}

fn field_name(name: &str) -> Ident {
    if is_invalid_name(name) {
        format!("`{}`", name)
    } else {
        name.to_owned()
    }
}

fn type_name(name: &str, used_names: &HashSet<String>) -> TypeName {
    let mut base_name = type_case(name.trim());
    if base_name.is_empty() {
        base_name = "GeneratedType".into();
    }
    let mut raw = base_name.clone();
    let mut n = 2;
    // will fail if name is sanitized
    while used_names.contains(&raw) {
        raw = format!("{}{}", base_name, n);
        n += 1;
    }
    let safe = sanitize_name(&raw);
    TypeName { raw, safe }
}

fn sanitize_name(name: &str) -> Ident {
    if is_invalid_name(name) {
        format!("`{}`", name)
    } else {
        name.to_owned()
    }
}

#[rustfmt::skip]
const RESERVED_WORDS: &[&str] = &[
    "package", "import",
    "object", "class", "trait",
    "sealed", "abstract", "case", "extends", "with",
    "private", "protected", "override", "implicit",
    "def", "val", "var", "lazy", "type",
    "while", "if", "else", "for", "yield", "try", "catch", "return", "throw",
    "new", "null", "this", "true", "false"
];

#[rustfmt::skip]
const RESERVED_CHARS: &[char] = &[
    '(', ')', '{', '}', '[', ']',  // parentheses
    '\'', '"', '.', ';', ',', // delimiter
    ' ', '=', '@', '#',  // other
];

// Scala is very flexible with it's naming and this is by no means complete
// For more see: https://scala-lang.org/files/archive/spec/2.13/01-lexical-syntax.html
fn is_invalid_name(name: &str) -> bool {
    if let Some(c) = name.chars().next() {
        if c.is_ascii_digit() || (c == '_' && name.len() == 1) {
            return true;
        }
    }
    if name.chars().any(|c| RESERVED_CHARS.contains(&c)) {
        return true;
    }
    RESERVED_WORDS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_invalid_name_syntax() {
        assert!(!is_invalid_name("foo"));
        assert!(!is_invalid_name("Foo"));
        assert!(!is_invalid_name("fooBar"));
        assert!(!is_invalid_name("FooBar"));
        assert!(!is_invalid_name("foo_bar"));
        assert!(!is_invalid_name("Foo_Bar"));
        assert!(!is_invalid_name("FOO_BAR"));
        assert!(!is_invalid_name("foo2"));
        assert!(!is_invalid_name("foo_!"));

        assert!(is_invalid_name("foo bar"));
        assert!(is_invalid_name("[foo]"));
        assert!(is_invalid_name("#foo"));
        assert!(is_invalid_name("foo#"));
        assert!(is_invalid_name("@foo"));
        assert!(is_invalid_name("foo@"));
    }

    #[test]
    fn test_is_invalid_name_reserved() {
        assert!(is_invalid_name("private"));
        assert!(is_invalid_name("for"));
        assert!(is_invalid_name("case"));
        assert!(is_invalid_name("type"));

        assert!(!is_invalid_name("_type"));
        assert!(!is_invalid_name("__type"));
    }

    #[test]
    fn test_field_name() {
        assert!(field_name("foo") == "foo");
        assert!(field_name("type") == "`type`");
        assert!(field_name("foo bar") == "`foo bar`");
    }

    #[test]
    fn test_type_name() {
        let mut used = HashSet::new();
        assert!(type_name("foo", &used).safe == "Foo");
        assert!(type_name("type", &used).safe == "Type");
        assert!(type_name("foo bar", &used).safe == "FooBar");
        used.insert("FooBar".to_owned());
        assert!(type_name("foo_bar", &used).safe == "FooBar2");
        assert!(type_name("123", &used).safe == "`123`");
        used.insert("`123`".to_owned());
        assert!(type_name("123", &used).safe == "`123`");
    }
}

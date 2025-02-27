use linked_hash_map::LinkedHashMap;

use crate::generation::typescript::{collapse_option, is_ts_identifier};
use crate::options::Options;
use crate::shape::{self, common_shape, Shape};
use crate::util::lower_camel_case;

pub struct Ctxt {
    options: Options,
    indent_level: usize,
}

pub type Code = String;

pub fn zod_schema(name: &str, shape: &Shape, options: Options) -> Code {
    let mut ctxt = Ctxt {
        options,
        indent_level: 1,
    };

    let code = type_from_shape(&mut ctxt, shape);
    let mut schema_name = lower_camel_case(name);
    schema_name.push_str("Schema");

    format!("export const {} = {};\n\n", schema_name, code)
}

fn type_from_shape(ctxt: &mut Ctxt, shape: &Shape) -> Code {
    use crate::shape::Shape::*;
    match shape {
        Null | Any | Bottom => "z.unknown()".into(),
        Bool => "z.boolean()".into(),
        StringT => "z.string()".into(),
        Integer => "z.number()".into(),
        Floating => "z.number()".into(),
        Tuple(shapes, _n) => {
            let folded = shape::fold_shapes(shapes.clone());
            if folded == Any && shapes.iter().any(|s| s != &Any) {
                generate_tuple_type(ctxt, shapes)
            } else {
                generate_vec_type(ctxt, &folded)
            }
        }
        VecT { elem_type: e } => generate_vec_type(ctxt, e),
        Struct { fields } => {
            if ctxt.options.infer_map_threshold.is_some_and(|lim| { fields.len() > lim }) {
                let inner = fields
                    .into_iter()
                    .map(|(_, value)| value.clone())
                    .fold(Shape::Bottom, common_shape);
                generate_map_type(ctxt, &inner)
            } else {
                generate_struct_from_field_shapes(ctxt, fields)
            }
        }
        MapT { val_type: v } => generate_map_type(ctxt, v),
        Opaque(t) => t.clone(),
        Optional(e) => {
            let inner = type_from_shape(ctxt, e);
            if ctxt.options.use_default_for_missing_fields {
                inner
            } else {
                format!("{}.optional()", inner)
            }
        },
        Nullable(e) => {
            let inner = type_from_shape(ctxt, e);
            if ctxt.options.use_default_for_missing_fields {
                inner
            } else {
                format!("{}.nullable()", inner)
            }
        },
    }
}

fn generate_vec_type(ctxt: &mut Ctxt, shape: &Shape) -> Code {
    let inner = type_from_shape(ctxt, shape);
    format!("{}.array()", inner)
}

fn generate_map_type(ctxt: &mut Ctxt, shape: &Shape) -> Code {
    let (_was_optional, collapsed) = collapse_option(shape);
    let inner = type_from_shape(ctxt, collapsed);
    format!("z.record(z.string(), {})", inner)
}

fn generate_tuple_type(ctxt: &mut Ctxt, shapes: &[Shape]) -> Code {
    let mut types = Vec::new();

    for shape in shapes {
        let typ = type_from_shape(ctxt, shape);
        types.push(typ);
    }

    format!("z.tuple([{}])", types.join(", "))
}

fn generate_struct_from_field_shapes(ctxt: &mut Ctxt, map: &LinkedHashMap<String, Shape>) -> Code {
    let fields: Vec<Code> = map
        .iter()
        .map(|(name, typ)| {
            ctxt.indent_level += 1;
            let field_type = type_from_shape(ctxt, typ);
            ctxt.indent_level -= 1;

            let escape_name = !is_ts_identifier(name);

            format!(
                "{}{}{}{}: {};",
                "    ".repeat(ctxt.indent_level),
                if escape_name { "\"" } else { "" },
                name,
                if escape_name { "\"" } else { "" },
                field_type
            )
        })
        .collect();

    let mut code = "z.object({\n".to_string();

    if !fields.is_empty() {
        code += &fields.join("\n");
        code += "\n";
    }
    code += &"    ".repeat(ctxt.indent_level - 1);
    code += "})";

    code
}

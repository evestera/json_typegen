use inflector::Inflector;
use linked_hash_map::LinkedHashMap;
use serde_json::{json, Value};

use crate::options::Options;
use crate::shape::{self, Shape};

#[allow(dead_code)]
pub struct Ctxt {
    options: Options,
}

pub type Ident = String;
pub type Code = String;

pub fn json_schema(name: &str, shape: &Shape, options: Options) -> (Ident, Option<Code>) {
    let mut ctxt = Ctxt { options };

    let ident = "".to_string();
    let value = type_from_shape(&mut ctxt, name, shape);

    let mut schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": format!("Generated schema for {}", name)
    });

    if let Value::Object(map) = value {
        for (key, val) in map.into_iter() {
            schema[key] = val;
        }
    }

    let code = ::serde_json::to_string_pretty(&schema);

    (ident, code.ok())
}

fn type_from_shape(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> Value {
    use crate::shape::Shape::*;
    match shape {
        Null | Any | Bottom => json!({}),
        Bool => json!({ "type": "boolean" }),
        StringT => json!({ "type": "string" }),
        Integer => json!({ "type": "number" }),
        Floating => json!({ "type": "number" }),
        Tuple(shapes, _n) => {
            let folded = shape::fold_shapes(shapes.clone());
            if folded == Any && shapes.iter().any(|s| s != &Any) {
                generate_tuple_type(ctxt, path, &shapes)
            } else {
                generate_vec_type(ctxt, path, &folded)
            }
        }
        VecT { elem_type: e } => generate_vec_type(ctxt, path, &e),
        Struct { fields: map } => generate_struct_from_field_shapes(ctxt, path, &map),
        MapT { val_type: v } => generate_map_type(ctxt, path, &v),
        Opaque(t) => json!({ "type": t }),
        Optional(e) => type_from_shape(ctxt, path, &e),
    }
}

fn generate_vec_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> Value {
    let singular = path.to_singular();
    let inner = type_from_shape(ctxt, &singular, shape);
    json!({
        "type": "array",
        "items": inner
    })
}

fn generate_map_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> Value {
    let singular = path.to_singular();
    let inner = type_from_shape(ctxt, &singular, shape);
    json!({
        "type": "object",
        "additionalProperties": inner
    })
}

fn generate_tuple_type(ctxt: &mut Ctxt, path: &str, shapes: &[Shape]) -> Value {
    let mut types = Vec::new();

    for shape in shapes {
        let typ = type_from_shape(ctxt, path, shape);
        types.push(typ);
    }

    json!({
        "type": "array",
        "items": types,
        "additionalItems": false
    })
}

fn collapse_option(typ: &Shape) -> (bool, &Shape) {
    if let Shape::Optional(inner) = typ {
        return (true, &**inner);
    }
    (false, typ)
}

fn generate_struct_from_field_shapes(
    ctxt: &mut Ctxt,
    _path: &str,
    map: &LinkedHashMap<String, Shape>,
) -> Value {
    let mut required: Vec<String> = Vec::new();
    let mut properties = json!({});

    for (name, typ) in map.iter() {
        let (was_optional, collapsed) = collapse_option(typ);

        if !was_optional {
            required.push(name.clone());
        }

        let field_code = type_from_shape(ctxt, name, collapsed);

        properties[name] = field_code;
    }

    json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

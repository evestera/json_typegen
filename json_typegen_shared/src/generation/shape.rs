use inflector::Inflector;
use linked_hash_map::LinkedHashMap;
use serde_json::{json, Value};

use crate::options::Options;
use crate::shape::{self, Shape};

#[allow(dead_code)]
pub struct Ctxt {
    options: Options,
}

pub type Code = String;

pub fn shape_string(name: &str, shape: &Shape, options: Options) -> Code {
    let mut ctxt = Ctxt { options };

    let value = type_from_shape(&mut ctxt, name, shape);

    ::serde_json::to_string_pretty(&value).unwrap()
}

fn type_from_shape(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> Value {
    use crate::shape::Shape::*;
    match *shape {
        Null => Value::Null,
        Any => json!("any"),
        Bottom => json!("bottom"),
        Bool => json!("bool"),
        StringT => json!("string"),
        Integer => json!("integer"),
        Floating => json!("floating"),
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
        Opaque(ref t) => json!(t),
        Optional(ref e) => json!({
            "__type__": "optional",
            "item": type_from_shape(ctxt, path, e),
        }),
    }
}

fn generate_vec_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> Value {
    let singular = path.to_singular();
    let inner = type_from_shape(ctxt, &singular, shape);
    json!([inner])
}

fn generate_map_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> Value {
    let singular = path.to_singular();
    let inner = type_from_shape(ctxt, &singular, shape);
    json!({
        "__type__": "map",
        "values": inner
    })
}

fn generate_tuple_type(ctxt: &mut Ctxt, path: &str, shapes: &[Shape]) -> Value {
    let mut types = Vec::new();

    for shape in shapes {
        let typ = type_from_shape(ctxt, path, shape);
        types.push(typ);
    }

    json!({
        "__type__": "tuple",
        "items": types,
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
    let mut properties = json!({});

    for (name, typ) in map.iter() {
        let (was_optional, collapsed) = collapse_option(typ);

        let annotated_name = if was_optional {
            name.to_owned() + "?"
        } else {
            name.to_owned()
        };

        let field_code = type_from_shape(ctxt, name, collapsed);

        properties[annotated_name] = field_code;
    }

    properties
}

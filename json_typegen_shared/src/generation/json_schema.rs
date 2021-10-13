use inflector::Inflector;
use linked_hash_map::LinkedHashMap;

use crate::generation::value::{pretty_print_value, Value};
use crate::options::Options;
use crate::shape::{self, Shape};
use crate::util::string_hashmap;

#[allow(dead_code)]
pub struct Ctxt {
    options: Options,
}

pub type Code = String;

pub fn json_schema(name: &str, shape: &Shape, options: Options) -> Code {
    let mut ctxt = Ctxt { options };

    let value = type_from_shape(&mut ctxt, name, shape);

    let mut schema = string_hashmap! {
        "$schema" => Value::Str("http://json-schema.org/draft-07/schema#"),
        "title" => Value::String(format!("Generated schema for {}", name)),
    };

    if let Value::Object(map) = value {
        for (key, val) in map.into_iter() {
            schema.insert(key, val);
        }
    }

    pretty_print_value(0, &Value::Object(schema))
}

fn type_from_shape(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> Value {
    use crate::shape::Shape::*;
    match shape {
        Null | Any | Bottom => Value::Object(LinkedHashMap::new()),
        Bool => Value::Object(string_hashmap! { "type" => Value::Str("boolean") }),
        StringT => Value::Object(string_hashmap! { "type" => Value::Str("string") }),
        Integer => Value::Object(string_hashmap! { "type" => Value::Str("number") }),
        Floating => Value::Object(string_hashmap! { "type" => Value::Str("number") }),
        Tuple(shapes, _n) => {
            let folded = shape::fold_shapes(shapes.clone());
            if folded == Any && shapes.iter().any(|s| s != &Any) {
                generate_tuple_type(ctxt, path, shapes)
            } else {
                generate_vec_type(ctxt, path, &folded)
            }
        }
        VecT { elem_type: e } => generate_vec_type(ctxt, path, e),
        Struct { fields: map } => generate_struct_from_field_shapes(ctxt, path, map),
        MapT { val_type: v } => generate_map_type(ctxt, path, v),
        Opaque(t) => Value::Object(string_hashmap! { "type" => Value::String(t.clone()) }),
        Optional(e) => type_from_shape(ctxt, path, e),
    }
}

fn generate_vec_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> Value {
    let singular = path.to_singular();
    let inner = type_from_shape(ctxt, &singular, shape);
    Value::Object(string_hashmap! {
        "type" => Value::Str("array"),
        "items" => inner
    })
}

fn generate_map_type(ctxt: &mut Ctxt, path: &str, shape: &Shape) -> Value {
    let singular = path.to_singular();
    let inner = type_from_shape(ctxt, &singular, shape);
    Value::Object(string_hashmap! {
        "type" => Value::Str("object"),
        "additionalProperties" => inner
    })
}

fn generate_tuple_type(ctxt: &mut Ctxt, path: &str, shapes: &[Shape]) -> Value {
    let mut types = Vec::new();

    for shape in shapes {
        let typ = type_from_shape(ctxt, path, shape);
        types.push(typ);
    }

    Value::Object(string_hashmap! {
        "type" => Value::Str("array"),
        "items" => Value::Array(types),
        "additionalItems" => Value::Bool(false)
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
    let mut required: Vec<Value> = Vec::new();
    let mut properties = LinkedHashMap::new();

    for (name, typ) in map.iter() {
        let (was_optional, collapsed) = collapse_option(typ);

        if !was_optional {
            required.push(Value::String(name.clone()));
        }

        let field_code = type_from_shape(ctxt, name, collapsed);

        properties.insert(name.to_string(), field_code);
    }

    Value::Object(string_hashmap! {
        "type" => Value::Str("object"),
        "properties" => Value::Object(properties),
        "required" => Value::Array(required)
    })
}

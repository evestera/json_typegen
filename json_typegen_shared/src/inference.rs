use serde_json::{ Value, Map };

use hints::{Hints, HintType};
use shape::{Shape, common_shape};

pub fn value_to_shape(value: &Value, hints: &Hints) -> Shape {
    for hint in hints.applicable.iter() {
        match hint.hint_type {
            HintType::MapType(_) => {
                if let Value::Object(ref map) = *value {
                    hint.used.set(true);
                    return object_to_map_shape(map, hints);
                } else {
                    panic!("Hint use_type map used on invalid value {:?}", value);
                }
            }
            HintType::OpaqueType(ref t) => {
                return Shape::Opaque(t.clone());
            }
            _ => {}
        }
    }

    match *value {
        Value::Null => Shape::Null,
        Value::Bool(_) => Shape::Bool,
        Value::Number(ref n) => {
            if n.is_i64() {
                Shape::Integer
            } else {
                Shape::Floating
            }
        },
        Value::String(_) => Shape::StringT,
        Value::Array(ref values) => array_to_shape(values, hints),
        Value::Object(ref map) => object_to_struct_shape(map, hints),
    }
}

fn array_to_shape(values: &[Value], hints: &Hints) -> Shape {
    if values.len() > 1 && values.len() <= 12 {
        let shapes: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(i, val)| value_to_shape(val, &hints.step_index(i)))
            .collect();
        return Shape::Tuple(shapes, 1);
    }
    let inner = values.iter().fold(Shape::Bottom, |shape, val| {
        let shape2 = value_to_shape(val, &hints.step_array());
        common_shape(shape, shape2)
    });
    Shape::VecT { elem_type: Box::new(inner) }
}

fn object_to_struct_shape(map: &Map<String, Value>, hints: &Hints) -> Shape {
    let inner = map.iter()
        .map(|(name, value)| {
            (name.clone(), value_to_shape(value, &hints.step_field(&name)))
        })
        .collect();
    Shape::Struct { fields: inner }
}

fn object_to_map_shape(map: &Map<String, Value>, hints: &Hints) -> Shape {
    let inner = map.values().fold(Shape::Bottom, |shape, val| {
        let shape2 = value_to_shape(val, &hints.step_any());
        common_shape(shape, shape2)
    });
    Shape::MapT { val_type: Box::new(inner) }
}


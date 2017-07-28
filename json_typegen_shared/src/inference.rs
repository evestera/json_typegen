use linked_hash_map::LinkedHashMap;
use serde_json::{ Value, Map };

use hints::*;

#[derive(Debug, PartialEq, Clone)]
pub enum Shape {
    Null,
    Any,
    Bottom,
    Bool,
    StringT,
    Integer,
    Floating,
    VecT { elem_type: Box<Shape> },
    Struct { fields: LinkedHashMap<String, Shape> },
    Tuple(Vec<Shape>, u64),
    MapT { val_type: Box<Shape> },
    Optional(Box<Shape>)
}

pub fn fold_shapes(shapes: Vec<Shape>) -> Shape {
    shapes.into_iter().fold(Shape::Bottom, common_shape)
}

fn common_shape(a: Shape, b: Shape) -> Shape {
    if a == b {
        return a;
    }
    use self::Shape::*;
    match (a, b) {
        (a, Bottom) | (Bottom, a) => a,
        (Integer, Floating) |
        (Floating, Integer) => Floating,
        (a, Null) | (Null, a) => make_optional(a),
        (a, Optional(b)) | (Optional(b), a) => make_optional(common_shape(a, *b)),
        (Tuple(shapes1, n1), Tuple(shapes2, n2)) => {
            if shapes1.len() == shapes2.len() {
                let shapes: Vec<_> = shapes1.into_iter()
                                            .zip(shapes2.into_iter())
                                            .map(|(a, b)| common_shape(a, b))
                                            .collect();
                Tuple(shapes, n1 + n2)
            } else {
                VecT {
                    elem_type: Box::new(common_shape(fold_shapes(shapes1), fold_shapes(shapes2)))
                }
            }
        }
        (Tuple(shapes, _), VecT { elem_type: e1 }) |
        (VecT { elem_type: e1 }, Tuple(shapes, _)) => {
            VecT { elem_type: Box::new(common_shape(*e1, fold_shapes(shapes))) }
        }
        (VecT { elem_type: e1 }, VecT { elem_type: e2 }) => {
            VecT { elem_type: Box::new(common_shape(*e1, *e2)) }
        }
        (MapT { val_type: v1 }, MapT { val_type: v2 }) => {
            MapT { val_type: Box::new(common_shape(*v1, *v2)) }
        }
        (Struct { fields: f1 }, Struct { fields: f2 }) => {
            Struct { fields: common_field_shapes(f1, f2) }
        }
        _ => Any,
    }
}

fn make_optional(a: Shape) -> Shape {
    use self::Shape::*;
    match a {
        Null | Any | Bottom | Optional(_) => a,
        non_nullable => Optional(Box::new(non_nullable)),
    }
}

fn common_field_shapes(f1: LinkedHashMap<String, Shape>,
                       mut f2: LinkedHashMap<String, Shape>)
                       -> LinkedHashMap<String, Shape> {
    if f1 == f2 {
        return f1;
    }
    let mut unified = LinkedHashMap::new();
    for (key, val) in f1.into_iter() {
        match f2.remove(&key) {
            Some(val2) => {
                unified.insert(key, common_shape(val, val2));
            },
            None => {
                unified.insert(key, make_optional(val));
            }
        }
    }
    for (key, val) in f2.into_iter() {
        unified.insert(key, make_optional(val));
    }
    unified
}

pub fn value_to_shape(value: &Value, hints: &Hints) -> Shape {
    for hint in hints.applicable.iter() {
        match hint.hint_type {
            HintType::UseMap(_) => {
                if let Value::Object(ref map) = *value {
                    hint.used.set(true);
                    return object_to_map_shape(map, hints);
                } else {
                    panic!("Hint use_type map used on invalid value {:?}", value);
                }
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

#[test]
fn test_unify() {
    use Shape::*;
    assert_eq!(common_shape(Bool, Bool), Bool);
    assert_eq!(common_shape(Bool, Integer), Any);
    assert_eq!(common_shape(Integer, Floating), Floating);
    assert_eq!(common_shape(Null, Any), Any);
    assert_eq!(common_shape(Null, Bool), Optional(Box::new(Bool)));
    assert_eq!(common_shape(Null, Optional(Box::new(Integer))), Optional(Box::new(Integer)));
    assert_eq!(common_shape(Any, Optional(Box::new(Integer))), Any);
    assert_eq!(common_shape(Any, Optional(Box::new(Integer))), Any);
    assert_eq!(common_shape(Optional(Box::new(Integer)), Optional(Box::new(Floating))),
               Optional(Box::new(Floating)));
    assert_eq!(common_shape(Optional(Box::new(StringT)), Optional(Box::new(Integer))), Any);
}

// based on hashmap! macro from maplit crate
#[cfg(test)]
macro_rules! string_hashmap {
    ($($key:expr => $value:expr,)+) => { string_hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let mut _map = ::linked_hash_map::LinkedHashMap::new();
            $(
                _map.insert($key.to_string(), $value);
            )*
            _map
        }
    };
}

#[test]
fn test_common_field_shapes() {
    use Shape::*;
    {
        let f1 = string_hashmap!{
            "a" => Integer,
            "b" => Bool,
            "c" => Integer,
            "d" => StringT,
        };
        let f2 = string_hashmap!{
            "a" => Integer,
            "c" => Floating,
            "d" => Null,
            "e" => Any,
        };
        assert_eq!(common_field_shapes(f1, f2), string_hashmap!{
            "a" => Integer,
            "b" => Optional(Box::new(Bool)),
            "c" => Floating,
            "d" => Optional(Box::new(StringT)),
            "e" => Any,
        });
    }
}

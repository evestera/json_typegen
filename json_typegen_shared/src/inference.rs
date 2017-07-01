use linked_hash_map::LinkedHashMap;
use serde_json::{ Value, Map };

#[derive(Debug, PartialEq, Clone)]
pub enum InferredType {
    Null,
    Any,
    Bottom,
    Bool,
    StringT,
    Integer,
    Floating,
    VecT { elem_type: Box<InferredType> },
    Struct { fields: LinkedHashMap<String, InferredType> },
    Optional(Box<InferredType>)
}

fn unify(a: InferredType, b: InferredType) -> InferredType {
    if a == b {
        return a;
    }
    use self::InferredType::*;
    match (a, b) {
        (a, Bottom) | (Bottom, a) => a,
        (Integer, Floating) |
        (Floating, Integer) => Floating,
        (a, Null) | (Null, a) => make_optional(a),
        (a, Optional(b)) | (Optional(b), a) => make_optional(unify(a, *b)),
        (VecT { elem_type: e1 }, VecT { elem_type: e2 }) => {
            VecT { elem_type: Box::new(unify(*e1, *e2)) }
        }
        (Struct { fields: f1 }, Struct { fields: f2 }) => {
            Struct { fields: unify_struct_fields(f1, f2) }
        }
        _ => Any,
    }
}

fn make_optional(a: InferredType) -> InferredType {
    use self::InferredType::*;
    match a {
        Null | Any | Bottom | Optional(_) => a,
        non_nullable => Optional(Box::new(non_nullable)),
    }
}

fn unify_struct_fields(f1: LinkedHashMap<String, InferredType>,
                       mut f2: LinkedHashMap<String, InferredType>)
                       -> LinkedHashMap<String, InferredType> {
    if f1 == f2 {
        return f1;
    }
    let mut unified = LinkedHashMap::new();
    for (key, val) in f1.into_iter() {
        match f2.remove(&key) {
            Some(val2) => {
                unified.insert(key, unify(val, val2));
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

pub fn infer_type_from_value(value: &Value) -> InferredType {
    match *value {
        Value::Null => InferredType::Null,
        Value::Bool(_) => InferredType::Bool,
        Value::Number(ref n) => {
            if n.is_i64() {
                InferredType::Integer
            } else {
                InferredType::Floating
            }
        },
        Value::String(_) => InferredType::StringT,
        Value::Array(ref values) => {
            infer_type_for_array(values)
        },
        Value::Object(ref map) => {
            InferredType::Struct { fields: infer_types_for_fields(map) }
        }
    }
}

fn infer_type_for_array(values: &[Value]) -> InferredType {
    let inner = values.iter().fold(InferredType::Bottom, |typ, val| {
        let new_type = infer_type_from_value(val);
        unify(typ, new_type)
    });
    InferredType::VecT { elem_type: Box::new(inner) }
}

fn infer_types_for_fields(map: &Map<String, Value>) -> LinkedHashMap<String, InferredType> {
    map.iter()
        .map(|(name, value)| (name.clone(), infer_type_from_value(value)))
        .collect()
}

#[test]
fn test_unify() {
    use InferredType::*;
    assert_eq!(unify(Bool, Bool), Bool);
    assert_eq!(unify(Bool, Integer), Any);
    assert_eq!(unify(Integer, Floating), Floating);
    assert_eq!(unify(Null, Any), Any);
    assert_eq!(unify(Null, Bool), Optional(Box::new(Bool)));
    assert_eq!(unify(Null, Optional(Box::new(Integer))), Optional(Box::new(Integer)));
    assert_eq!(unify(Any, Optional(Box::new(Integer))), Any);
    assert_eq!(unify(Any, Optional(Box::new(Integer))), Any);
    assert_eq!(unify(Optional(Box::new(Integer)), Optional(Box::new(Floating))),
               Optional(Box::new(Floating)));
    assert_eq!(unify(Optional(Box::new(StringT)), Optional(Box::new(Integer))), Any);
}

// based on hashmap! macro from maplit crate
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
fn test_unify_struct_fields() {
    use InferredType::*;
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
        assert_eq!(unify_struct_fields(f1, f2), string_hashmap!{
            "a" => Integer,
            "b" => Optional(Box::new(Bool)),
            "c" => Floating,
            "d" => Optional(Box::new(StringT)),
            "e" => Any,
        });
    }
}

use linked_hash_map::LinkedHashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum Shape {
    /// `Bottom` represents the absence of any inference information
    Bottom,

    /// `Any` represents conflicting inference information that can not be
    /// represented by any single shape
    Any,

    /// `Optional(T)` represents that a value is nullable, or not always present
    Optional(Box<Shape>),

    /// Equivalent to `Optional(Bottom)`, `Null` represents optionality with no further information
    Null,

    Bool,
    StringT,
    Integer,
    Floating,
    VecT { elem_type: Box<Shape> },
    Struct { fields: LinkedHashMap<String, Shape> },
    Tuple(Vec<Shape>, u64),
    MapT { val_type: Box<Shape> },
    Opaque(String),
}

pub fn fold_shapes(shapes: Vec<Shape>) -> Shape {
    shapes.into_iter().fold(Shape::Bottom, common_shape)
}

pub fn common_shape(a: Shape, b: Shape) -> Shape {
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
        },
        (Opaque(t), _) | (_, Opaque(t)) => Opaque(t),
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

#[test]
fn test_unify() {
    use self::Shape::*;
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
    use self::Shape::*;
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

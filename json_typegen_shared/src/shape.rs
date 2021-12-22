use linked_hash_map::LinkedHashMap;

/// The type representing the inferred structure
///
/// A word of caution: Everything in this crate is "internal API", but for this type in particular,
/// since it is very central to how json_typegen works,
/// be prepared that major breaking changes may need to be made to this in the future.
#[non_exhaustive]
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
    VecT {
        elem_type: Box<Shape>,
    },
    Struct {
        fields: LinkedHashMap<String, Shape>,
    },
    Tuple(Vec<Shape>, u64),
    MapT {
        val_type: Box<Shape>,
    },
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
        (Integer, Floating) | (Floating, Integer) => Floating,
        (a, Null) | (Null, a) => a.into_optional(),
        (a, Optional(b)) | (Optional(b), a) => common_shape(a, *b).into_optional(),
        (Tuple(shapes1, n1), Tuple(shapes2, n2)) => {
            if shapes1.len() == shapes2.len() {
                let shapes: Vec<_> = shapes1
                    .into_iter()
                    .zip(shapes2.into_iter())
                    .map(|(a, b)| common_shape(a, b))
                    .collect();
                Tuple(shapes, n1 + n2)
            } else {
                VecT {
                    elem_type: Box::new(common_shape(fold_shapes(shapes1), fold_shapes(shapes2))),
                }
            }
        }
        (Tuple(shapes, _), VecT { elem_type: e1 }) | (VecT { elem_type: e1 }, Tuple(shapes, _)) => {
            VecT {
                elem_type: Box::new(common_shape(*e1, fold_shapes(shapes))),
            }
        }
        (VecT { elem_type: e1 }, VecT { elem_type: e2 }) => VecT {
            elem_type: Box::new(common_shape(*e1, *e2)),
        },
        (MapT { val_type: v1 }, MapT { val_type: v2 }) => MapT {
            val_type: Box::new(common_shape(*v1, *v2)),
        },
        (Struct { fields: f1 }, Struct { fields: f2 }) => Struct {
            fields: common_field_shapes(f1, f2),
        },
        (Opaque(t), _) | (_, Opaque(t)) => Opaque(t),
        _ => Any,
    }
}

fn common_field_shapes(
    mut f1: LinkedHashMap<String, Shape>,
    mut f2: LinkedHashMap<String, Shape>,
) -> LinkedHashMap<String, Shape> {
    if f1 == f2 {
        return f1;
    }
    for (key, val) in f1.iter_mut() {
        let temp = std::mem::replace(val, Shape::Bottom);
        match f2.remove(key) {
            Some(val2) => {
                *val = common_shape(temp, val2);
            }
            None => {
                *val = temp.into_optional();
            }
        };
    }
    for (key, val) in f2.into_iter() {
        f1.insert(key, val.into_optional());
    }
    f1
}

impl Shape {
    fn into_optional(self) -> Self {
        use self::Shape::*;
        match self {
            Null | Any | Bottom | Optional(_) => self,
            non_nullable => Optional(Box::new(non_nullable)),
        }
    }

    /// Note: This is asymmetrical because we don't unify based on this,
    /// but check if `self` can be used *as is* as a replacement for `other`
    pub(crate) fn is_acceptable_substitution_for(&self, other: &Shape) -> bool {
        use self::Shape::*;
        if self == other {
            return true;
        }
        match (self, other) {
            (_, Bottom) => true,
            (Optional(_), Null) => true,
            (Optional(a), Optional(b)) => a.is_acceptable_substitution_for(b),
            (VecT { elem_type: e1 }, VecT { elem_type: e2 }) => {
                e1.is_acceptable_substitution_for(e2)
            }
            (MapT { val_type: v1 }, MapT { val_type: v2 }) => v1.is_acceptable_substitution_for(v2),
            (Tuple(a, _), Tuple(b, _)) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b.iter())
                        .all(|(e1, e2)| e1.is_acceptable_substitution_for(e2))
            }
            (Struct { fields: f1 }, Struct { fields: f2}) => {
                // Require all fields to be the same (but ignore order)
                // Could maybe be more lenient, e.g. for missing optional fields
                f1.len() == f2.len() && f1.iter().all(|(key, shape1)| {
                    if let Some(shape2) = f2.get(key) {
                        shape1.is_acceptable_substitution_for(shape2)
                    } else {
                        false
                    }
                })
            }
            _ => false,
        }
    }
}

#[test]
fn test_unify() {
    use self::Shape::*;
    assert_eq!(common_shape(Bool, Bool), Bool);
    assert_eq!(common_shape(Bool, Integer), Any);
    assert_eq!(common_shape(Integer, Floating), Floating);
    assert_eq!(common_shape(Null, Any), Any);
    assert_eq!(common_shape(Null, Bool), Optional(Box::new(Bool)));
    assert_eq!(
        common_shape(Null, Optional(Box::new(Integer))),
        Optional(Box::new(Integer))
    );
    assert_eq!(common_shape(Any, Optional(Box::new(Integer))), Any);
    assert_eq!(common_shape(Any, Optional(Box::new(Integer))), Any);
    assert_eq!(
        common_shape(Optional(Box::new(Integer)), Optional(Box::new(Floating))),
        Optional(Box::new(Floating))
    );
    assert_eq!(
        common_shape(Optional(Box::new(StringT)), Optional(Box::new(Integer))),
        Any
    );
}

#[test]
fn test_common_field_shapes() {
    use self::Shape::*;
    use crate::util::string_hashmap;
    {
        let f1 = string_hashmap! {
            "a" => Integer,
            "b" => Bool,
            "c" => Integer,
            "d" => StringT,
        };
        let f2 = string_hashmap! {
            "a" => Integer,
            "c" => Floating,
            "d" => Null,
            "e" => Any,
        };
        assert_eq!(
            common_field_shapes(f1, f2),
            string_hashmap! {
                "a" => Integer,
                "b" => Optional(Box::new(Bool)),
                "c" => Floating,
                "d" => Optional(Box::new(StringT)),
                "e" => Any,
            }
        );
    }
}

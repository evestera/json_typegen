use crate::Options;
use crate::hints::{HintType, Hints};
use crate::inference::jsoninputerr::JsonInputErr;
use crate::inference::jsonlex::{JsonLexer, JsonToken};
use crate::shape::{Shape, common_shape};
use linked_hash_map::LinkedHashMap;
use std::io::Read;
use std::iter::Peekable;

pub fn shape_from_json<R: Read>(
    read: R,
    options: &Options,
    hints: &Hints,
) -> Result<Shape, JsonInputErr> {
    let pointer = options.unwrap.clone();
    let pointer_tokens: Vec<&str> = if pointer.is_empty() || pointer == "/" {
        vec![]
    } else if pointer.starts_with('/') {
        pointer.split('/').skip(1).collect()
    } else {
        pointer.split('/').collect()
    };

    Inference::new(read)
        .unwrap(&pointer_tokens, options, hints)?
        .ok_or(JsonInputErr::NoMatchForUnwrap)
}

struct Inference<T: Iterator<Item = Result<JsonToken, JsonInputErr>>> {
    tokens: Peekable<T>,
}

impl<R: Read> Inference<JsonLexer<R>> {
    fn new(source: R) -> Self {
        Inference {
            tokens: JsonLexer::new(source).peekable(),
        }
    }
}

impl<T: Iterator<Item = Result<JsonToken, JsonInputErr>>> Inference<T> {
    fn next_token(&mut self) -> Result<JsonToken, JsonInputErr> {
        match self.tokens.next() {
            Some(Ok(token)) => Ok(token),
            Some(Err(err)) => Err(err),
            None => Err(JsonInputErr::UnexpectedEndOfInput),
        }
    }

    fn expect_token(&mut self, expected_token: JsonToken) -> Result<(), JsonInputErr> {
        if self.next_token()? == expected_token {
            Ok(())
        } else {
            Err(JsonInputErr::InvalidJson)
        }
    }

    fn infer_shape(&mut self, options: &Options, hints: &Hints) -> Result<Shape, JsonInputErr> {
        for hint in hints.applicable.iter() {
            match hint.hint_type {
                HintType::MapType(_) => {
                    return if let JsonToken::ObjectStart = self.next_token()? {
                        hint.used.set(true);
                        self.infer_map(options, hints)
                    } else {
                        Err(JsonInputErr::InvalidTargetForHint)
                    };
                }
                HintType::OpaqueType(ref t) => {
                    hint.used.set(true);
                    let _ = self.infer_shape(options, &Hints::new()); // parse and discard the actual value
                    return Ok(Shape::Opaque(t.clone()));
                }
                _ => {}
            }
        }

        match self.next_token()? {
            JsonToken::True | JsonToken::False => Ok(Shape::Bool),
            JsonToken::Null => Ok(Shape::Null),
            JsonToken::Number(s) => {
                if s.contains('.') {
                    Ok(Shape::Floating)
                } else {
                    Ok(Shape::Integer)
                }
            }
            JsonToken::String(_) => Ok(Shape::StringT),
            JsonToken::ObjectStart => self.infer_object(options, hints),
            JsonToken::ArrayStart => self.infer_array(options, hints),
            JsonToken::ObjectEnd | JsonToken::ArrayEnd | JsonToken::Comma | JsonToken::Colon => {
                Err(JsonInputErr::InvalidJson)
            }
        }
    }

    fn infer_map(&mut self, options: &Options, hints: &Hints) -> Result<Shape, JsonInputErr> {
        let shape = self.infer_object(options, hints)?;
        match shape {
            Shape::Struct { fields } => {
                let inner = fields
                    .into_iter()
                    .map(|(_, value)| value)
                    .fold(Shape::Bottom, common_shape);
                Ok(Shape::MapT {
                    val_type: Box::new(inner),
                })
            }
            _ => {
                panic!("Got non-object from infer_object")
            }
        }
    }

    fn infer_object(&mut self, options: &Options, hints: &Hints) -> Result<Shape, JsonInputErr> {
        if let Some(&Ok(JsonToken::ObjectEnd)) = self.tokens.peek() {
            self.tokens.next();
            return Ok(Shape::Struct {
                fields: LinkedHashMap::new(),
            });
        }

        let mut fields = LinkedHashMap::new();
        loop {
            let token = self.next_token()?;

            let key = match token {
                JsonToken::String(s) => s,
                _ => return Err(JsonInputErr::InvalidJson),
            };

            self.expect_token(JsonToken::Colon)?;

            let value = self.infer_shape(options, &hints.step_field(&key))?;
            fields.insert(key, value);

            if let Some(&Ok(JsonToken::ObjectEnd)) = self.tokens.peek() {
                self.tokens.next();
                break;
            }

            self.expect_token(JsonToken::Comma)?;
        }

        if options
            .infer_map_threshold
            .is_some_and(|lim| fields.len() > lim)
        {
            let inner = fields
                .into_iter()
                .map(|(_, value)| value)
                .fold(Shape::Bottom, common_shape);
            Ok(Shape::MapT {
                val_type: Box::new(inner),
            })
        } else {
            Ok(Shape::Struct { fields })
        }
    }

    fn infer_array(&mut self, options: &Options, hints: &Hints) -> Result<Shape, JsonInputErr> {
        if let Some(&Ok(JsonToken::ArrayEnd)) = self.tokens.peek() {
            self.tokens.next();
            return Ok(Shape::VecT {
                elem_type: Box::new(Shape::Bottom),
            });
        }

        let mut len = 0;
        let mut shapes: Vec<Shape> = vec![];
        let mut folded = Shape::Bottom;

        loop {
            if len < 12 {
                let shape = self.infer_shape(options, &hints.step_index(len))?;
                shapes.push(shape);
            } else {
                let shape = self.infer_shape(options, &hints.step_array())?;
                folded = common_shape(shape, folded);
            }
            len += 1;

            if let Some(&Ok(JsonToken::ArrayEnd)) = self.tokens.peek() {
                self.tokens.next();
                break;
            }

            self.expect_token(JsonToken::Comma)?;
        }

        if len > 1 && len <= 12 {
            return Ok(Shape::Tuple(shapes, 1));
        }

        let inner = shapes.into_iter().fold(folded, common_shape);

        Ok(Shape::VecT {
            elem_type: Box::new(inner),
        })
    }

    /// "Unwrap" JSON nodes before doing inference
    ///
    /// The node(s) specified by the pointer is the new root(s) that we do inference on
    fn unwrap(
        &mut self,
        pointer_tokens: &[&str],
        options: &Options,
        hints: &Hints,
    ) -> Result<Option<Shape>, JsonInputErr> {
        let (first_token, rest_of_pointer) = match pointer_tokens.split_first() {
            None => {
                return Ok(Some(self.infer_shape(options, hints)?));
            }
            Some(val) => val,
        };

        match self.next_token()? {
            JsonToken::True
            | JsonToken::Null
            | JsonToken::False
            | JsonToken::Number(_)
            | JsonToken::String(_) => Ok(None),
            JsonToken::ObjectStart => {
                if let Some(&Ok(JsonToken::ObjectEnd)) = self.tokens.peek() {
                    self.tokens.next();
                    return Ok(None);
                }

                let mut folded = None;
                loop {
                    let token = self.next_token()?;

                    let key = match token {
                        JsonToken::String(s) => s,
                        _ => return Err(JsonInputErr::InvalidJson),
                    };

                    self.expect_token(JsonToken::Colon)?;

                    if *first_token == "-" || *first_token == key {
                        let result = self.unwrap(rest_of_pointer, options, hints)?;
                        folded = optional_common_shape(folded, result);
                    } else {
                        // parse and discard non-matched element (could use non-inference code)
                        let _ = self.infer_shape(options, hints)?;
                    }

                    if let Some(&Ok(JsonToken::ObjectEnd)) = self.tokens.peek() {
                        self.tokens.next();
                        return Ok(folded);
                    }

                    self.expect_token(JsonToken::Comma)?;
                }
            }
            JsonToken::ArrayStart => {
                if let Some(&Ok(JsonToken::ArrayEnd)) = self.tokens.peek() {
                    self.tokens.next();
                    return Ok(None);
                }

                let first_token_is_numeric =
                    first_token.bytes().all(|b| (b'0'..=b'9').contains(&b));

                let mut folded = None;

                for index in 0.. {
                    if *first_token == "-"
                        || (first_token_is_numeric && *first_token == index.to_string())
                    {
                        let result = self.unwrap(rest_of_pointer, options, hints)?;
                        folded = optional_common_shape(folded, result);
                    } else {
                        // parse and discard non-matched element (could use non-inference code)
                        let _ = self.infer_shape(options, hints)?;
                    }

                    if let Some(&Ok(JsonToken::ArrayEnd)) = self.tokens.peek() {
                        self.tokens.next();
                        break;
                    }

                    self.expect_token(JsonToken::Comma)?;
                }

                Ok(folded)
            }
            JsonToken::ObjectEnd | JsonToken::ArrayEnd | JsonToken::Comma | JsonToken::Colon => {
                Err(JsonInputErr::InvalidJson)
            }
        }
    }
}

fn optional_common_shape(a: Option<Shape>, b: Option<Shape>) -> Option<Shape> {
    match (a, b) {
        (Some(a), Some(b)) => Some(common_shape(a, b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::string_hashmap;

    fn shape_from_just_json<R: Read>(json: R) -> Result<Shape, JsonInputErr> {
        Inference::new(json).infer_shape(&Options::default(), &Hints::new())
    }

    #[test]
    fn infer_object() {
        assert_eq!(
            shape_from_just_json(r#"{}"#.as_bytes()),
            Ok(Shape::Struct {
                fields: string_hashmap!()
            })
        );
        assert_eq!(
            shape_from_just_json(
                r#"{
                    "foo": true
                }"#
                .as_bytes()
            ),
            Ok(Shape::Struct {
                fields: string_hashmap! {
                "foo" => Shape::Bool,
                }
            })
        );
        assert_eq!(
            shape_from_just_json(
                r#"{
                    "foo": true,
                    "bar": false
                }"#
                .as_bytes()
            ),
            Ok(Shape::Struct {
                fields: string_hashmap! {
                    "foo" => Shape::Bool,
                    "bar" => Shape::Bool,
                }
            })
        );

        assert_eq!(
            shape_from_just_json(
                r#"{
                    "foo": true
                    "bar": false
                }"#
                .as_bytes()
            ),
            Err(JsonInputErr::InvalidJson)
        );
        assert_eq!(
            shape_from_just_json(
                r#"{
                    "foo": true,
                }"#
                .as_bytes()
            ),
            Err(JsonInputErr::InvalidJson)
        );
        assert_eq!(
            shape_from_just_json(
                r#"{
                    "foo": true,
                "#
                .as_bytes()
            ),
            Err(JsonInputErr::UnexpectedEndOfInput)
        );
    }

    #[test]
    fn infer_array() {
        assert_eq!(
            shape_from_just_json(r#"[]"#.as_bytes()),
            Ok(Shape::VecT {
                elem_type: Box::new(Shape::Bottom)
            })
        );
        assert_eq!(
            shape_from_just_json(r#"[true]"#.as_bytes()),
            Ok(Shape::VecT {
                elem_type: Box::new(Shape::Bool)
            })
        );
        assert_eq!(
            shape_from_just_json(r#"[true, false]"#.as_bytes()),
            Ok(Shape::Tuple(vec![Shape::Bool, Shape::Bool], 1)) // flattened in a later step
        );
        assert_eq!(
            shape_from_just_json(r#"[true, "hello"]"#.as_bytes()),
            Ok(Shape::Tuple(vec![Shape::Bool, Shape::StringT], 1))
        );

        assert_eq!(
            shape_from_just_json(r#"[true false]"#.as_bytes()),
            Err(JsonInputErr::InvalidJson)
        );
        assert_eq!(
            shape_from_just_json(r#"[true,]"#.as_bytes()),
            Err(JsonInputErr::InvalidJson)
        );
        assert_eq!(
            shape_from_just_json(r#"[true"#.as_bytes()),
            Err(JsonInputErr::UnexpectedEndOfInput)
        );
    }

    fn unwrap_test(json: &str, pointer: &str, result: Result<Shape, JsonInputErr>) {
        let shape = shape_from_json(
            json.as_bytes(),
            &Options {
                unwrap: pointer.into(),
                ..Options::default()
            },
            &Hints::new(),
        );

        assert_eq!(shape, result);
    }

    #[test]
    fn no_unwrap() {
        unwrap_test(
            r#"{ "foo": 5 }"#,
            "",
            Ok(Shape::Struct {
                fields: string_hashmap! {
                    "foo" => Shape::Integer,
                },
            }),
        );
    }

    #[test]
    fn unwrap_object_wildcard() {
        unwrap_test(r#"{ "foo": 5 }"#, "/-", Ok(Shape::Integer));
    }

    #[test]
    fn unwrap_object_key() {
        unwrap_test(r#"{ "foo": 5, "bar": "baz" }"#, "/foo", Ok(Shape::Integer));
    }

    #[test]
    fn unwrap_array_wildcard() {
        unwrap_test(r#"[5, 6]"#, "/-", Ok(Shape::Integer));
    }

    #[test]
    fn unwrap_array_index() {
        unwrap_test(r#"["foo", 6]"#, "/1", Ok(Shape::Integer));
    }
}

use crate::inference::jsoninputerr::JsonInputErr;
use crate::inference::jsonlex::{JsonLexer, JsonToken};
use crate::shape::{common_shape, Shape};
use linked_hash_map::LinkedHashMap;
use std::io::Read;
use std::iter::Peekable;

pub trait FromJson {
    fn from_json(json: impl Read) -> Result<Self, JsonInputErr>
    where
        Self: Sized;
}

impl FromJson for Shape {
    fn from_json(json: impl Read) -> Result<Self, JsonInputErr> {
        Inference::new(json).infer_value()
    }
}

struct Inference<R: Read> {
    tokens: Peekable<JsonLexer<R>>,
}

impl<R: Read> Inference<R> {
    fn new(source: R) -> Self {
        Inference {
            tokens: JsonLexer::new(source).peekable(),
        }
    }

    fn next_token(&mut self) -> Result<JsonToken, JsonInputErr> {
        match self.tokens.next() {
            Some(Ok(token)) => Ok(token),
            Some(Err(err)) => Err(err),
            None => Err(JsonInputErr::UnexpectedEndOfInput),
        }
    }

    fn infer_value(&mut self) -> Result<Shape, JsonInputErr> {
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
            JsonToken::ObjectStart => self.infer_object(),
            JsonToken::ArrayStart => self.infer_array(),
            JsonToken::ObjectEnd | JsonToken::ArrayEnd | JsonToken::Comma | JsonToken::Colon => {
                Err(JsonInputErr::InvalidJson)
            }
        }
    }

    fn infer_object(&mut self) -> Result<Shape, JsonInputErr> {
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

            let value = self.infer_value()?;
            fields.insert(key, value);

            if let Some(&Ok(JsonToken::ObjectEnd)) = self.tokens.peek() {
                self.tokens.next();
                return Ok(Shape::Struct { fields });
            }

            self.expect_token(JsonToken::Comma)?;
        }
    }

    fn expect_token(&mut self, expected_token: JsonToken) -> Result<(), JsonInputErr> {
        if self.next_token()? == expected_token {
            Ok(())
        } else {
            Err(JsonInputErr::InvalidJson)
        }
    }

    fn infer_array(&mut self) -> Result<Shape, JsonInputErr> {
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
            let shape = self.infer_value()?;
            len += 1;
            if len <= 12 {
                shapes.push(shape);
            } else {
                folded = common_shape(shape, folded);
            }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::string_hashmap;

    #[test]
    fn infer_object() {
        assert_eq!(
            Shape::from_json(r#"{}"#.as_bytes()),
            Ok(Shape::Struct {
                fields: string_hashmap!()
            })
        );
        assert_eq!(
            Shape::from_json(
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
            Shape::from_json(
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
            Shape::from_json(
                r#"{
                    "foo": true
                    "bar": false
                }"#
                .as_bytes()
            ),
            Err(JsonInputErr::InvalidJson)
        );
        assert_eq!(
            Shape::from_json(
                r#"{
                    "foo": true,
                }"#
                .as_bytes()
            ),
            Err(JsonInputErr::InvalidJson)
        );
        assert_eq!(
            Shape::from_json(
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
            Shape::from_json(r#"[]"#.as_bytes()),
            Ok(Shape::VecT {
                elem_type: Box::new(Shape::Bottom)
            })
        );
        assert_eq!(
            Shape::from_json(r#"[true]"#.as_bytes()),
            Ok(Shape::VecT {
                elem_type: Box::new(Shape::Bool)
            })
        );
        assert_eq!(
            Shape::from_json(r#"[true, false]"#.as_bytes()),
            Ok(Shape::Tuple(vec![Shape::Bool, Shape::Bool], 1)) // flattened in a later step
        );
        assert_eq!(
            Shape::from_json(r#"[true, "hello"]"#.as_bytes()),
            Ok(Shape::Tuple(vec![Shape::Bool, Shape::StringT], 1))
        );

        assert_eq!(
            Shape::from_json(r#"[true false]"#.as_bytes()),
            Err(JsonInputErr::InvalidJson)
        );
        assert_eq!(
            Shape::from_json(r#"[true,]"#.as_bytes()),
            Err(JsonInputErr::InvalidJson)
        );
        assert_eq!(
            Shape::from_json(r#"[true"#.as_bytes()),
            Err(JsonInputErr::UnexpectedEndOfInput)
        );
    }
}

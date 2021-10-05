use crate::inference::jsoninputerr::JsonInputErr;
use std::io::{BufReader, Bytes, Read};
use std::iter::Peekable;

#[derive(Debug, PartialEq)]
pub enum JsonToken {
    True,
    False,
    Null,
    Number(String),
    String(String),
    ObjectStart,
    ObjectEnd,
    ArrayStart,
    ArrayEnd,
    Comma,
    Colon,
}

pub struct JsonLexer<R: Read> {
    bytes: Peekable<Bytes<BufReader<R>>>, // TODO: Keep position info
    failed: bool,
}

impl<R: Read> JsonLexer<R> {
    pub fn new(source: R) -> Self {
        JsonLexer {
            bytes: BufReader::new(source).bytes().peekable(),
            failed: false,
        }
    }

    fn get_next_token(&mut self) -> Option<Result<JsonToken, JsonInputErr>> {
        loop {
            let byte = match self.bytes.peek() {
                Some(Ok(byte)) => *byte,
                Some(Err(_)) => return Some(Err(JsonInputErr::IoErr)),
                None => return None,
            };

            return Some(match byte {
                b' ' | b'\t' | b'\n' | b'\r' => {
                    self.bytes.next();
                    continue;
                }
                b'{' => self.skip_and_produce(JsonToken::ObjectStart),
                b'}' => self.skip_and_produce(JsonToken::ObjectEnd),
                b'[' => self.skip_and_produce(JsonToken::ArrayStart),
                b']' => self.skip_and_produce(JsonToken::ArrayEnd),
                b',' => self.skip_and_produce(JsonToken::Comma),
                b':' => self.skip_and_produce(JsonToken::Colon),
                b't' => self.match_token("true", JsonToken::True),
                b'f' => self.match_token("false", JsonToken::False),
                b'n' => self.match_token("null", JsonToken::Null),
                b'"' => self.match_string(),
                b'0'..=b'9' | b'-' => self.match_number(),
                _ => Err(JsonInputErr::InvalidJson),
            });
        }
    }

    fn skip_and_produce(&mut self, token: JsonToken) -> Result<JsonToken, JsonInputErr> {
        self.bytes.next();
        Ok(token)
    }

    fn expect_byte(&mut self) -> Result<u8, JsonInputErr> {
        match self.bytes.next() {
            Some(Ok(byte)) => Ok(byte),
            Some(Err(_)) => Err(JsonInputErr::IoErr),
            None => Err(JsonInputErr::UnexpectedEndOfInput),
        }
    }

    fn skip_byte(&mut self, target_byte: u8) -> Result<(), JsonInputErr> {
        let byte = self.expect_byte()?;
        if byte == target_byte {
            Ok(())
        } else {
            Err(JsonInputErr::InvalidJson)
        }
    }

    fn match_token(
        &mut self,
        target_str: &'static str,
        token: JsonToken,
    ) -> Result<JsonToken, JsonInputErr> {
        for target_byte in target_str.bytes() {
            self.skip_byte(target_byte)?;
        }
        Ok(token)
    }

    fn match_string(&mut self) -> Result<JsonToken, JsonInputErr> {
        self.skip_byte(b'"')?;
        let mut buffer = Vec::new();
        loop {
            let byte = self.expect_byte()?;

            if byte == b'"' {
                return Ok(JsonToken::String(
                    String::from_utf8(buffer).map_err(|_| JsonInputErr::InvalidUtf8)?,
                ));
            } else if byte == b'\\' {
                let escaped = self.expect_byte()?;

                match escaped {
                    b'"' => buffer.push(b'"'),
                    b'\\' => buffer.push(b'\\'),
                    b'/' => buffer.push(b'/'),
                    b'b' => buffer.push(8),  // backspace
                    b'f' => buffer.push(12), // form feed
                    b'n' => buffer.push(b'\n'),
                    b'r' => buffer.push(b'\r'),
                    b't' => buffer.push(b'\t'),
                    b'u' => {
                        let surrogate_offset: u32 = (0xD800 << 10) + 0xDC00 - 0x10000;

                        let mut codepoint = self.parse_codepoint()? as u32;
                        if (0xD800..=0xDFFF).contains(&codepoint) {
                            // first codepoint was the start of a surrogate pair
                            self.skip_byte(b'\\')?;
                            self.skip_byte(b'u')?;
                            let codepoint2 = self.parse_codepoint()? as u32;
                            codepoint = ((codepoint << 10) + codepoint2) - surrogate_offset;
                        };
                        let mut buf = [0u8; 4];
                        let encoded_bytes = std::char::from_u32(codepoint)
                            .unwrap_or(std::char::REPLACEMENT_CHARACTER)
                            .encode_utf8(&mut buf)
                            .bytes();
                        for encoded_byte in encoded_bytes {
                            buffer.push(encoded_byte);
                        }
                    }
                    _ => return Err(JsonInputErr::InvalidEscape(escaped)),
                };
            } else {
                buffer.push(byte)
            }
        }
    }

    // "ab03..." -> 0xab03
    fn parse_codepoint(&mut self) -> Result<u16, JsonInputErr> {
        let mut codepoint: u16 = 0;
        for _ in 0..4 {
            codepoint <<= 4;
            let byte2 = self.expect_byte()?;
            codepoint += match byte2 {
                b'0'..=b'9' => byte2 - b'0',
                b'a'..=b'f' => byte2 - b'a' + 10,
                b'A'..=b'F' => byte2 - b'A' + 10,
                _ => return Err(JsonInputErr::InvalidEscape(byte2)),
            } as u16;
        }
        Ok(codepoint)
    }

    fn match_number(&mut self) -> Result<JsonToken, JsonInputErr> {
        let mut buffer = Vec::new();
        loop {
            let byte = match self.bytes.peek() {
                Some(Ok(byte)) => *byte,
                Some(Err(_)) => return Err(JsonInputErr::IoErr),
                None => break,
            };

            match byte {
                b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E' => {
                    buffer.push(byte);
                    self.bytes.next();
                }
                _ => break,
            }
        }
        // TODO: Actually parse numbers
        Ok(JsonToken::Number(
            String::from_utf8(buffer).map_err(|_err| JsonInputErr::InvalidUtf8)?,
        ))
    }
}

impl<R: Read> Iterator for JsonLexer<R> {
    type Item = Result<JsonToken, JsonInputErr>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }

        let res = self.get_next_token();
        if let Some(Err(_)) = res {
            self.failed = true;
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inference::jsoninputerr::JsonInputErr;
    use std::fmt::Debug;

    #[test]
    fn empty_input() {
        assert_eq!(tokens_from_str(""), Ok(vec![]));
        assert_eq!(tokens_from_str(" \t\r\n"), Ok(vec![]));
    }

    #[test]
    fn bare_number() {
        assert_eq!(
            tokens_from_str("123"),
            Ok(vec![JsonToken::Number("123".to_string())])
        );
    }

    #[test]
    fn object() {
        assert_eq!(
            tokens_from_str("{}"),
            Ok(vec![JsonToken::ObjectStart, JsonToken::ObjectEnd])
        );
        assert_eq!(
            tokens_from_str("  {  }  "),
            Ok(vec![JsonToken::ObjectStart, JsonToken::ObjectEnd])
        );
    }

    #[test]
    fn string() {
        assert_eq!(
            tokens_from_str(r#" "hello world" "#),
            Ok(vec![JsonToken::String("hello world".to_string())])
        );
    }

    #[test]
    fn escapes() {
        assert_eq!(
            tokens_from_str(r#" "foo\nbar" "#),
            Ok(vec![JsonToken::String("foo\nbar".to_string())])
        );

        assert_eq!(
            tokens_from_str(r#" "John says \"Hello\"" "#),
            Ok(vec![JsonToken::String(r#"John says "Hello""#.to_string())])
        );
    }

    #[test]
    fn unicode_escapes() {
        assert_eq!(
            tokens_from_str(r#" "\u00e6" "#),
            Ok(vec![JsonToken::String("æ".to_string())])
        );

        assert_eq!(
            tokens_from_str(r#" "\uD83D\uDE00" "#),
            Ok(vec![JsonToken::String("😀".to_string())])
        );

        assert_eq!(
            tokens_from_str(r#" "\uD83D" "#),
            Err(JsonInputErr::InvalidJson)
        );
    }

    #[test]
    fn number() {
        assert_eq!(
            tokens_from_str(r#" 14.5 "#),
            Ok(vec![JsonToken::Number("14.5".to_string())])
        );
    }

    #[test]
    fn complex() {
        assert_eq!(
            tokens_from_str(
                r#"
                {
                    "foo": [1, true]
                }
                "#
            ),
            Ok(vec![
                JsonToken::ObjectStart,
                JsonToken::String("foo".to_string()),
                JsonToken::Colon,
                JsonToken::ArrayStart,
                JsonToken::Number("1".to_string()),
                JsonToken::Comma,
                JsonToken::True,
                JsonToken::ArrayEnd,
                JsonToken::ObjectEnd
            ])
        );
    }

    #[test]
    fn invalid() {
        assert_eq!(tokens_from_str("foo"), Err(JsonInputErr::InvalidJson));
        assert_eq!(tokens_from_str(" [ foo ] "), Err(JsonInputErr::InvalidJson));
    }

    fn tokens_from_str(s: &'static str) -> Result<Vec<JsonToken>, JsonInputErr> {
        let collected: Vec<Result<JsonToken, JsonInputErr>> =
            JsonLexer::new(s.as_bytes()).collect();
        coalesce_err(collected)
    }

    fn coalesce_err<T: Debug, E: Debug>(vec: Vec<Result<T, E>>) -> Result<Vec<T>, E> {
        let error_count = vec.iter().filter(|res| res.is_err()).count();
        match error_count {
            0 => Ok(vec.into_iter().map(|res| res.unwrap()).collect()),
            1 => Err(vec
                .into_iter()
                .find(|res| res.is_err())
                .unwrap()
                .unwrap_err()),
            _ => panic!("More than one error: {:?}", vec),
        }
    }
}

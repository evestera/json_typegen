use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum JsonInputErr {
    #[error("IO-related error")]
    IoErr,
    #[error("Sample contained invalid UTF-8")]
    InvalidUtf8,
    #[error("Sample contained invalid JSON")]
    InvalidJson,
    #[error("Sample contained an invalid escape")]
    InvalidEscape(u8),
    #[error("Reached end of input while parsing")]
    UnexpectedEndOfInput,
}

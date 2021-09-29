use std::fmt::{Display, Formatter};

/// The errors that json_typegen_shared may produce
///
/// No stability guarantees are made with for this type
/// except that it is a type that implements `std::error::Error`
#[non_exhaustive]
#[derive(Debug)]
pub enum JTError {
    #[cfg(feature = "remote-samples")]
    SampleFetchingError(reqwest::Error),
    #[cfg(feature = "local-samples")]
    SampleReadingError(std::io::Error),
    JsonParsingError(serde_json::Error),
    MacroParsingError(String),
}

impl Display for JTError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "remote-samples")]
            JTError::SampleFetchingError(_) => {
                write!(f, "An error occurred while fetching JSON")
            }
            #[cfg(feature = "local-samples")]
            JTError::SampleReadingError(_) => {
                write!(f, "An error occurred while reading JSON from file")
            }
            JTError::JsonParsingError(_) => {
                write!(f, "An error occurred while parsing JSON")
            }
            JTError::MacroParsingError(msg) => {
                write!(
                    f,
                    "An error occurred while parsing a macro or macro input: {}",
                    msg
                )
            }
        }
    }
}

impl std::error::Error for JTError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(feature = "remote-samples")]
            JTError::SampleFetchingError(e) => Some(e),
            #[cfg(feature = "local-samples")]
            JTError::SampleReadingError(e) => Some(e),
            JTError::JsonParsingError(e) => Some(e),
            JTError::MacroParsingError(_) => None,
        }
    }
}

#[cfg(feature = "remote-samples")]
impl From<reqwest::Error> for JTError {
    fn from(err: reqwest::Error) -> Self {
        JTError::SampleFetchingError(err)
    }
}

#[cfg(feature = "local-samples")]
impl From<std::io::Error> for JTError {
    fn from(err: std::io::Error) -> Self {
        JTError::SampleReadingError(err)
    }
}

impl From<serde_json::Error> for JTError {
    fn from(err: serde_json::Error) -> Self {
        JTError::JsonParsingError(err)
    }
}

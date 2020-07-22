/// Represents an error that occur during parsing.
#[derive(thiserror::Error, Debug)]
#[error("ads.txt parse error: {0}")]
pub struct Error(&'static str);

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error(s)
    }
}

/// Wrapper for the `Result` type with an [`Error`](struct.Error.html).
pub type Result<T> = std::result::Result<T, Error>;

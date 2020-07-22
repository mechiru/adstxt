/// Represents an error that occur during crawling.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("build request error: {0}")]
    Request(#[from] hyper::http::Error),
    #[error("header encoding error: {0}")]
    HeaderEncoding(#[from] hyper::http::header::ToStrError),
    #[error("body encoding error: {0}")]
    BodyEncoding(#[from] std::string::FromUtf8Error),
    #[error("ads.txt crawle error: {0}")]
    Crawle(#[from] hyper::Error),
    #[error("task execution error: {0}")]
    Task(#[from] tokio::task::JoinError),
}

/// Wrapper for the `Result` type with an [`Error`](enum.Error.html).
pub type Result<T> = std::result::Result<T, Error>;

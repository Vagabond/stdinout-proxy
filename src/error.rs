use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("axum error: {0}")]
    Axum(#[from] axum::BoxError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("environmental variable PROXY_EXEC is not set!")]
    NoExec,
}

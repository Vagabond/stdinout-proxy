use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("axum error: {0}")]
    Axum(#[from] axum::BoxError),
}

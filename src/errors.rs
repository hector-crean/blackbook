#[derive(thiserror::Error, Debug)]
pub enum BlackbookServerError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    AxumError(#[from] axum::Error),
    #[error(transparent)]
    EnvVariableError(#[from] std::env::VarError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error),
}

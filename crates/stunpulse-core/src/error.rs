use bb8::RunError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("JSON Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("UTF-8 Encoding Error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Postgres Error: {0}")]
    Postgres(#[from] tokio_postgres::Error),
    #[error("Postgres Connection Error: {0}")]
    PostgresConnection(#[from] RunError<tokio_postgres::Error>),
    #[error("Environment Variable Error: {0}")]
    EnvironmentVariable(#[from] std::env::VarError),
    #[error("WASM Error: {0}")]
    Wasm(#[from] wasmtime::Error),
}

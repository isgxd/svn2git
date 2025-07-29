use thiserror::Error;

/// 导出错误类型
pub type Result<T> = std::result::Result<T, SyncError>;

/// 错误类型
#[derive(Debug, Error)]
pub enum SyncError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to convert bytes to Utf8 string: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error("Invalid Utf8 in string slice: {0}")]
    StrUtf8(#[from] core::str::Utf8Error),

    #[error("Application error: {0}")]
    App(String),

    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Inquire error: {0}")]
    Inquire(#[from] inquire::error::InquireError),

    #[error("Roxmltree error: {0}")]
    Roxmltree(#[from] roxmltree::Error),
}

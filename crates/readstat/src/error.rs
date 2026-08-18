use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),
    #[error("ReadStat: {0}")]
    ReadStat(String),
    #[error("unsupported format for path: {0}")]
    UnsupportedFormat(String),
    #[error(".sas7bcat is a format catalog and cannot be opened as a dataset")]
    CatalogIsNotDataset,
    #[error("import cancelled")]
    Cancelled,
    #[error("invalid path")]
    InvalidPath,
    #[error("arrow: {0}")]
    Arrow(String),
}

impl Error {
    pub fn msg(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

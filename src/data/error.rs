use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(std::io::Error),
    #[error("{0}")]
    Serde(serde_json::Error),
    #[cfg(feature = "polygon")]
    #[error("{0}")]
    Polygon(::polygon::errors::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}
#[cfg(feature = "polygon")]
impl From<::polygon::errors::Error> for Error {
    fn from(e: ::polygon::errors::Error) -> Self {
        Self::Polygon(e)
    }
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(std::io::Error),
    #[cfg(feature = "polygon")]
    #[error("{0}")]
    Polygon(::polygon::errors::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[cfg(feature = "polygon")]
impl From<::polygon::errors::Error> for Error {
    fn from(e: ::polygon::errors::Error) -> Self {
        Self::Polygon(e)
    }
}

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(std::io::Error),
    #[error("{0}")]
    Encode(rmp_serde::encode::Error),
    #[error("{0}")]
    Decode(rmp_serde::decode::Error),
    #[cfg(feature = "polygon")]
    #[error("{0}")]
    Polygon(::polygon::errors::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(e: rmp_serde::encode::Error) -> Self {
        Self::Encode(e)
    }
}

impl From<rmp_serde::decode::Error> for Error {
    fn from(e: rmp_serde::decode::Error) -> Self {
        Self::Decode(e)
    }
}

#[cfg(feature = "polygon")]
impl From<::polygon::errors::Error> for Error {
    fn from(e: ::polygon::errors::Error) -> Self {
        Self::Polygon(e)
    }
}

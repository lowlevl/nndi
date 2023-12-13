use thiserror::Error;

/// The error types that can occur when manipulating this crate.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Mdns(#[from] mdns_sd::Error),

    #[error("Machine hostname must be UTF-8 encoded: {0}")]
    Hostname(std::str::Utf8Error),

    #[error(transparent)]
    Frame(#[from] binrw::Error),
}

/// A handy [`std::result::Result`] type alias bounding the [`enum@Error`] struct as `E`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

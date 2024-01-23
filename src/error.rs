use thiserror::Error;

/// The error types that can occur when manipulating this crate.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Mdns(#[from] mdns_sd::Error),

    #[error(transparent)]
    Frame(#[from] binrw::Error),

    #[error(transparent)]
    Xml(#[from] quick_xml::DeError),

    #[error(transparent)]
    Codec(#[from] ffmpeg::Error),

    #[error("The channel was closed, and cannot accept data anymore")]
    ClosedChannel,

    #[error("The peer timed out while awaiting mandatory data")]
    Timeout(#[from] tokio::time::error::Elapsed),

    #[error("Unknown frame kind from packet header")]
    UnknownKind,
}

/// A handy [`std::result::Result`] type alias bounding the [`enum@Error`] struct as `E`.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

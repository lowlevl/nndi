use thiserror::Error;

/// The error types that can occur when manipulating this crate.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// mDNS error.
    #[error(transparent)]
    Mdns(#[from] mdns_sd::Error),

    /// Frame decoding error.
    #[error(transparent)]
    Frame(#[from] binrw::Error),

    /// XML format error.
    #[error(transparent)]
    Xml(#[from] quick_xml::DeError),

    /// Transcoding error.
    #[error(transparent)]
    Codec(#[from] ffmpeg::Error),

    /// The channel was closed.
    #[error("The channel was closed, and cannot accept data anymore")]
    ClosedChannel,

    /// The peer timed out.
    #[error("The peer timed out while awaiting mandatory data")]
    Timeout(#[from] tokio::time::error::Elapsed),

    /// The packet was unknown, or unsupported.
    #[error("Unknown frame kind from packet header")]
    UnknownKind,
}

/// A handy [`std::result::Result`] type alias bounding the [`enum@Error`] struct as `E`.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

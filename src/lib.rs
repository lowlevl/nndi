#![doc = include_str!("../README.md")]
//!

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    missing_docs,
    clippy::unwrap_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::undocumented_unsafe_blocks
)]

use std::time::Duration;

pub extern crate ffmpeg_next as ffmpeg;

const SERVICE_TYPE: &str = "_ndi._tcp.local.";
const SDK_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "~", env!("CARGO_PKG_NAME"));
const SDK_PLATFORM: &str = "unknown";
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(3);

fn hostname() -> String {
    let hostname = gethostname::gethostname();
    String::from_utf8_lossy(&hostname.into_encoded_bytes()).to_string()
}

fn name(source: &str) -> String {
    let mut hostname = hostname();
    hostname.make_ascii_uppercase();

    format!("{hostname} ({source})")
}

mod io;

mod error;
pub use error::{Error, Result};

mod scan;
pub use scan::Scan;

pub mod sink;
pub use sink::Sink;

pub mod source;
pub use source::Source;

pub mod metadata {
    //! Metadata entries for the NDI sources.

    pub use crate::io::frame::text::{EnabledStreams, Identify, Tally, Version, VideoQuality};
}

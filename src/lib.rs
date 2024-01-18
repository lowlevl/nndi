#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    clippy::unwrap_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::undocumented_unsafe_blocks
)]

pub extern crate ffmpeg_next as ffmpeg;

const SERVICE_TYPE: &str = "_ndi._tcp.local.";
const SDK_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "~", env!("CARGO_PKG_NAME"));
const SDK_PLATFORM: &str = "unknown";

fn hostname() -> String {
    let hostname = gethostname::gethostname();
    String::from_utf8_lossy(&hostname.into_encoded_bytes()).to_string()
}

fn name(source: &str) -> String {
    let mut hostname = hostname();
    hostname.make_ascii_uppercase();

    format!("{hostname} ({source})")
}

mod error;
pub use error::{Error, Result};

mod io;

pub mod metadata {
    pub use crate::io::frame::text::{EnabledStreams, Identify, Tally, Version, VideoQuality};
}

mod scan;
pub use scan::Scan;

pub mod sink;
pub use sink::Sink;

pub mod source;
pub use source::Source;

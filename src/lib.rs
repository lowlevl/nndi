#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(
    clippy::unwrap_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::undocumented_unsafe_blocks
)]

const SERVICE_TYPE: &str = "_ndi._tcp.local.";
const SDK_VERSION: &str = "5.6.0";
const SDK_PLATFORM: &str = "LINUX";

fn hostname() -> String {
    let hostname = gethostname::gethostname();
    String::from_utf8_lossy(&hostname.into_encoded_bytes()).to_string()
}

fn name(subname: &str) -> String {
    format!("{} ({subname})", hostname().to_ascii_uppercase())
}

mod error;
pub use error::{Error, Result};

pub mod io;

pub mod recv;
pub mod scan;
pub mod send;

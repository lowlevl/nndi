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

fn hostname() -> Result<String> {
    let hostname = gethostname::gethostname();
    String::from_utf8(hostname.into_encoded_bytes()).map_err(Error::Hostname)
}

fn name(subname: &str) -> Result<String> {
    Ok(format!("{} ({subname})", hostname()?.to_ascii_uppercase()))
}

mod error;
pub use error::{Error, Result};

mod frame;

pub mod msg;
pub mod recv;
pub mod scan;
pub mod send;

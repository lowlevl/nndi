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

mod error;
pub use error::{Error, Result};

pub mod io;

pub mod send;

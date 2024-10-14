use std::net::SocketAddr;

use derive_more::BitOr;
use serde::{Deserialize, Serialize};

use crate::Result;

pub type Block = super::Block<[u8; 8], binrw::NullString>;

#[derive(Debug, Serialize, Deserialize)]
pub enum Metadata {
    #[serde(rename = "ndi_version")]
    Version(Version),

    #[serde(rename = "ndi_identify")]
    Identify(Identify),

    #[serde(rename = "ndi_video")]
    Video(Video),

    #[serde(rename = "ndi_enabled_streams")]
    EnabledStreams(EnabledStreams),

    #[serde(rename = "ntk_conn_feedback")]
    ConnectionFeedback(ConnectionFeedback),

    #[serde(rename = "ndi_tally")]
    Tally(Tally),

    #[serde(rename = "ndi_tally_echo")]
    TallyEcho(Tally),
}

impl Metadata {
    pub fn from_block(block: &Block) -> Result<Self> {
        let mut text = std::io::Cursor::new(&block.data.0);

        Ok(quick_xml::de::from_reader::<_, Self>(&mut text)?)
    }

    pub fn to_block(&self) -> Block {
        let text = quick_xml::se::to_string(&self)
            .expect("Unable to serialize XML structure, should not be the case");

        Block::data(text)
    }
}

/// Metadata definition for _version_ in the protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    /// Version number of _text_ frames.
    #[serde(rename = "@text")]
    pub text: u16,

    /// Version number of _video_ frames.
    #[serde(rename = "@video")]
    pub video: u16,

    /// Version number of _audio_ frames.
    #[serde(rename = "@audio")]
    pub audio: u16,

    /// Version of the _SDK_.
    #[serde(rename = "@sdk")]
    pub sdk: String,

    /// Platform running the _SDK_.
    #[serde(rename = "@platform")]
    pub platform: String,
}

/// Metadata definition for _identification_ in the protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identify {
    /// The name of the peer.
    #[serde(rename = "@name")]
    pub name: String,
}

/// Metadata definition for _video_ in the protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    /// The requested _quality_ of the video stream.
    #[serde(rename = "@quality")]
    pub quality: VideoQuality,
}

/// Different video qualities available in the protocol.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoQuality {
    /// High definition video stream.
    #[default]
    High,

    /// Low definition video stream.
    Low,
}

/// Metadata definition for _enabled streams_ in the protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnabledStreams {
    /// Whether _text_ streams are supported.
    #[serde(rename = "@text")]
    pub text: bool,

    /// Whether _video_ streams are supported.
    #[serde(rename = "@video")]
    pub video: bool,

    /// Whether _audio_ streams are supported.
    #[serde(rename = "@audio")]
    pub audio: bool,

    /// Whether SpeedHQ skip-block is supported.
    #[serde(rename = "@shq_skip_block")]
    pub shq_skip_block: bool,

    /// Whether SpeedHQ short-DC is supported.
    #[serde(rename = "@shq_short_dc")]
    pub shq_short_dc: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionFeedback {
    pub connection: Connection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@addr")]
    pub addr: SocketAddr,
    #[serde(rename = "@state")]
    pub state: ConnectionState,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Up,
    Down,
}

/// Metadata definition for _tally_ in the protocol.
#[derive(Debug, Default, Clone, BitOr, Serialize, Deserialize)]
pub struct Tally {
    /// Whether we currently are _on program_.
    #[serde(rename = "@on_program")]
    pub on_program: bool,

    /// Whether we currently are _on preview_.
    #[serde(rename = "@on_preview")]
    pub on_preview: bool,
}

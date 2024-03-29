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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    #[serde(rename = "@text")]
    pub text: u16,
    #[serde(rename = "@video")]
    pub video: u16,
    #[serde(rename = "@audio")]
    pub audio: u16,
    #[serde(rename = "@sdk")]
    pub sdk: String,
    #[serde(rename = "@platform")]
    pub platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identify {
    #[serde(rename = "@name")]
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    #[serde(rename = "@quality")]
    pub quality: VideoQuality,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoQuality {
    #[default]
    High,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnabledStreams {
    #[serde(rename = "@text")]
    pub text: bool,
    #[serde(rename = "@video")]
    pub video: bool,
    #[serde(rename = "@audio")]
    pub audio: bool,
    #[serde(rename = "@shq_skip_block")]
    pub shq_skip_block: bool,
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

#[derive(Debug, Default, Clone, BitOr, Serialize, Deserialize)]
pub struct Tally {
    #[serde(rename = "@on_program")]
    pub on_program: bool,
    #[serde(rename = "@on_preview")]
    pub on_preview: bool,
}

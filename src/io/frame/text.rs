use std::net::SocketAddr;

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

    #[serde(other)]
    Unknown,
}

impl Metadata {
    pub fn from_block(block: &Block) -> Result<Self> {
        let mut text = std::io::Cursor::new(&block.data.0);

        Ok(quick_xml::de::from_reader::<_, Self>(&mut text)?)
    }

    pub fn to_block(&self) -> Result<Block> {
        let text = quick_xml::se::to_string(&self)?;

        Ok(Block::data(text))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "ndi_version")]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "ndi_identify")]
pub struct Identify {
    #[serde(rename = "@name")]
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "ndi_video")]
pub struct Video {
    #[serde(rename = "@quality")]
    pub quality: VideoQuality,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoQuality {
    High,
    Low,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "ndi_enabled_streams")]
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
#[serde(rename = "ntk_conn_feedback")]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "ndi_tally")]
pub struct Tally {
    #[serde(rename = "@on_program")]
    pub on_program: bool,
    #[serde(rename = "@on_preview")]
    pub on_preview: bool,
}

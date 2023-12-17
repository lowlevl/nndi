use serde::{Deserialize, Serialize};

pub type Pack = super::Pack<(), binrw::NullString>;

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

    #[serde(rename = "ndi_tally")]
    Tally(Tally),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "ndi_version")]
pub struct Version {
    #[serde(rename = "@text")]
    pub text: u8,
    #[serde(rename = "@video")]
    pub video: u8,
    #[serde(rename = "@audio")]
    pub audio: u8,
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
#[serde(rename = "ndi_tally")]
pub struct Tally {
    #[serde(rename = "@on_program")]
    pub on_program: bool,
    #[serde(rename = "@on_preview")]
    pub on_preview: bool,
}

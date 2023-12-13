use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Info {
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
    text: u8,
    #[serde(rename = "@video")]
    video: u8,
    #[serde(rename = "@audio")]
    audio: u8,
    #[serde(rename = "@sdk")]
    sdk: String,
    #[serde(rename = "@platform")]
    platform: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "ndi_identify")]
pub struct Identify {
    #[serde(rename = "@name")]
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "ndi_video")]

pub struct Video {
    #[serde(rename = "@quality")]
    quality: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "ndi_enabled_streams")]
pub struct EnabledStreams {
    #[serde(rename = "@text")]
    text: bool,
    #[serde(rename = "@video")]
    video: bool,
    #[serde(rename = "@audio")]
    audio: bool,
    #[serde(rename = "@shq_skip_block")]
    shq_skip_block: bool,
    #[serde(rename = "@shq_short_dc")]
    shq_short_dc: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "ndi_tally")]
pub struct Tally {
    #[serde(rename = "@on_program")]
    on_program: bool,
    #[serde(rename = "@on_preview")]
    on_preview: bool,
}

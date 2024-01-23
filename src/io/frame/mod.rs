use derive_more::From;
use strum::{EnumDiscriminants, FromRepr};

use crate::Result;

mod block;
pub use block::{Block, BytesEof};

pub mod audio;
pub mod text;
pub mod video;

#[derive(Debug, PartialEq, From, EnumDiscriminants)]
#[strum_discriminants(name(FrameKind))]
#[strum_discriminants(derive(FromRepr))]
#[strum_discriminants(repr(u16))]
pub enum Frame {
    Video(video::Block),
    Audio(audio::Block),
    Text(text::Block),
}

impl Frame {
    pub fn from_parts(kind: FrameKind, header: &[u8], data: &[u8]) -> Result<Self> {
        let frame = match kind {
            FrameKind::Video => Self::Video(Block::from_raw(header, data)?),
            FrameKind::Audio => Self::Audio(Block::from_raw(header, data)?),
            FrameKind::Text => Self::Text(Block::from_raw(header, data)?),
        };

        Ok(frame)
    }

    pub fn to_parts(&self) -> Result<(FrameKind, Vec<u8>, Vec<u8>)> {
        let (kind, (header, data)) = match self {
            Self::Video(block) => (FrameKind::Video, block.to_raw()?),
            Self::Audio(block) => (FrameKind::Audio, block.to_raw()?),
            Self::Text(block) => (FrameKind::Text, block.to_raw()?),
        };

        Ok((kind, header, data))
    }

    pub fn version() -> Self {
        Self::Text(
            text::Metadata::Version(text::Version {
                video: 5,
                audio: 4,
                text: 3,
                sdk: crate::SDK_VERSION.into(),
                platform: crate::SDK_PLATFORM.into(),
            })
            .to_block()
            .expect("Invalid block construction"),
        )
    }

    pub fn identify(name: &str) -> Self {
        Self::Text(
            text::Metadata::Identify(text::Identify {
                name: crate::name(name),
            })
            .to_block()
            .expect("Invalid block construction"),
        )
    }

    pub fn video_meta(quality: text::VideoQuality) -> Self {
        Self::Text(
            text::Metadata::Video(text::Video { quality })
                .to_block()
                .expect("Invalid block construction"),
        )
    }

    pub fn enabled_streams(video: bool, audio: bool) -> Self {
        Self::Text(
            text::Metadata::EnabledStreams(text::EnabledStreams {
                video,
                audio,
                text: true,
                shq_skip_block: false,
                shq_short_dc: false,
            })
            .to_block()
            .expect("Invalid block construction"),
        )
    }
}

impl FrameKind {
    pub fn version(&self) -> u16 {
        match self {
            Self::Video => 4,
            Self::Audio => 3,
            Self::Text => 1,
        }
    }
}

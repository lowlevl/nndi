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

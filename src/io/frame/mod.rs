use binrw::{BinRead, BinWrite};
use strum::EnumDiscriminants;

mod block;
pub use block::Block;

pub mod audio;
pub mod text;
pub mod video;

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(FrameType))]
#[strum_discriminants(derive(BinRead, BinWrite))]
#[strum_discriminants(brw(repr = u16))]
pub enum Frame {
    Video(video::Block),
    Audio(audio::Block),
    Text(text::Block),
}

impl FrameType {
    pub fn version(&self) -> u16 {
        match self {
            Self::Video => 5,
            Self::Audio => 3,
            Self::Text => 1,
        }
    }
}

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct BytesEof {
    #[br(parse_with = binrw::helpers::until_eof)]
    inner: Vec<u8>,
}

impl std::ops::Deref for BytesEof {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

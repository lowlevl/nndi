use binrw::{meta::ReadEndian, BinRead, BinWrite};

use super::Packet;
use crate::Result;

pub mod audio;
pub mod text;
pub mod video;

#[derive(Debug)]
pub enum Frame {
    Video(video::Block),
    Audio(audio::Block),
    Text(text::Block),
}

#[derive(Debug, BinRead, BinWrite)]
#[brw(repr = u16)]
pub enum FrameType {
    Video = 0,
    Audio,
    Text,
}

impl FrameType {
    pub fn version(&self) -> u16 {
        match self {
            Self::Video => 4,
            Self::Audio => 3,
            Self::Text => 1,
        }
    }
}

#[derive(Debug, Default)]
pub struct Block<H, D> {
    pub header: H,
    pub data: D,
}

impl<H, D> Block<H, D> {
    pub fn new(header: H, data: impl Into<D>) -> Self {
        Self {
            header,
            data: data.into(),
        }
    }
}

impl<H: Default, D> Block<H, D> {
    pub fn data(data: impl Into<D>) -> Self {
        Self {
            header: Default::default(),
            data: data.into(),
        }
    }
}

impl<H, D> Block<H, D>
where
    H: for<'a> BinRead<Args<'a> = ()> + ReadEndian,
    D: for<'a> BinRead<Args<'a> = ()> + ReadEndian,
{
    pub fn from_pkt(pkt: Packet) -> Result<Self> {
        Ok(Self {
            header: BinRead::read(&mut std::io::Cursor::new(
                &pkt.data[..pkt.header_size as usize],
            ))?,
            data: BinRead::read(&mut std::io::Cursor::new(
                &pkt.data[pkt.header_size as usize..],
            ))?,
        })
    }
}

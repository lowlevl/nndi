use binrw::{meta::ReadEndian, BinRead, BinWrite};

use super::Pkt;
use crate::Result;

pub mod audio;
pub mod metadata;
pub mod video;

#[derive(Debug)]
pub enum Frame {
    Video(video::Pack),
    Audio(audio::Pack),
    Text(metadata::Pack),
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
pub struct Pack<H, D> {
    pub header: H,
    pub data: D,
}

impl<H, D> Pack<H, D> {
    pub fn new(header: H, data: impl Into<D>) -> Self {
        Self {
            header,
            data: data.into(),
        }
    }
}

impl<H: Default, D> Pack<H, D> {
    pub fn data(data: impl Into<D>) -> Self {
        Self {
            header: Default::default(),
            data: data.into(),
        }
    }
}

impl<H, D> Pack<H, D>
where
    H: for<'a> BinRead<Args<'a> = ()> + ReadEndian,
    D: for<'a> BinRead<Args<'a> = ()> + ReadEndian,
{
    pub fn from_frame(frame: Pkt) -> Result<Self> {
        Ok(Self {
            header: BinRead::read(&mut std::io::Cursor::new(
                &frame.data[..frame.header_size as usize],
            ))?,
            data: BinRead::read(&mut std::io::Cursor::new(
                &frame.data[frame.header_size as usize..],
            ))?,
        })
    }
}

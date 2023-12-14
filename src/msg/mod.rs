use binrw::{meta::ReadEndian, BinRead};

use crate::{frame::Frame, Result};

pub mod audio;
pub mod metadata;
pub mod video;

#[derive(Debug, Default)]
pub struct Payload<H, D> {
    pub header: H,
    pub data: D,
}

impl<H, D> Payload<H, D> {
    pub fn new(header: H, data: impl Into<D>) -> Self {
        Self {
            header,
            data: data.into(),
        }
    }
}

impl<H, D> Payload<H, D>
where
    H: for<'a> BinRead<Args<'a> = ()> + ReadEndian,
    D: for<'a> BinRead<Args<'a> = ()> + ReadEndian,
{
    pub fn from_frame(frame: Frame) -> Result<Self> {
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

impl<H: Default, D> Payload<H, D> {
    pub fn data(data: impl Into<D>) -> Self {
        Self {
            header: Default::default(),
            data: data.into(),
        }
    }
}

#[derive(Debug)]
pub enum Msg {
    Video(Payload<video::VideoSpec, binrw::NullString>),
    Audio(Payload<audio::AudioSpec, binrw::NullString>),
    Text(Payload<[u8; 8], binrw::NullString>),
}

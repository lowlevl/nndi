use binrw::{meta::ReadEndian, BinRead};

use crate::{frame::Frame, Result};

pub mod audio;
pub mod metadata;
pub mod video;

#[derive(Debug)]
pub enum Msg {
    Video(Wrap<video::Spec, binrw::NullString>),
    Audio(Wrap<audio::Spec, binrw::NullString>),
    Text(Wrap<(), binrw::NullString>),
}

#[derive(Debug, Default)]
pub struct Wrap<H, D> {
    pub header: H,
    pub data: D,
}

impl<H, D> Wrap<H, D> {
    pub fn new(header: H, data: impl Into<D>) -> Self {
        Self {
            header,
            data: data.into(),
        }
    }
}

impl<H: Default, D> Wrap<H, D> {
    pub fn data(data: impl Into<D>) -> Self {
        Self {
            header: Default::default(),
            data: data.into(),
        }
    }
}

impl<H, D> Wrap<H, D>
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

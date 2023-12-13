use binrw::{BinRead, BinWrite};

mod scrambler;
pub use scrambler::Scrambler;

use crate::{msg::Msg, Result};

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct Frame {
    /// The version of the frame, for retro-compatibility purposes.
    #[br(map(|version: u16| version & 0x7fff))] // FIXME: Handle when the MSB is `0`
    pub version: u16,

    /// The type of the frame, either [`FrameType::Video`], [`FrameType::Audio`] or [`FrameType::Text`].
    pub frame_type: FrameType,

    /// Size of the header in the data segment.
    pub header_size: u32,

    /// Size of the payload, after the header, in the data segment.
    pub payload_len: u32,

    /// The payload of the frame.
    #[br(count = header_size + payload_len)]
    pub data: Vec<u8>,
}

impl Frame {
    pub fn unpack(mut self) -> Result<Msg> {
        let scrambler = Scrambler::detect(&self);

        match self.frame_type {
            FrameType::Text => {
                scrambler.unscramble(&mut self.data[..], self.header_size + self.payload_len)
            }
            _ => scrambler.unscramble(
                &mut self.data[..self.header_size as usize],
                self.header_size + self.payload_len,
            ),
        }

        let mut data = std::io::Cursor::new(self.data);

        Ok(match self.frame_type {
            FrameType::Video => Msg::Video(BinRead::read(&mut data)?),
            FrameType::Audio => Msg::Audio(BinRead::read(&mut data)?),
            FrameType::Text => Msg::Text(BinRead::read(&mut data)?),
        })
    }
}

#[derive(Debug, BinRead, BinWrite)]
#[brw(repr(u16))]
pub enum FrameType {
    Video = 0,
    Audio,
    Text,
}

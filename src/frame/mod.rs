use binrw::{BinRead, BinWrite};

use crate::{
    msg::{Msg, Pack},
    Result,
};

mod scrambler;
pub use scrambler::Scrambler;

mod stream;
pub use stream::Stream;

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct Frame {
    /// The version of the frame, for retro-compatibility purposes.
    /// May need to account for the MSB being `0`.
    #[br(map(|version: u16| version & 0x7fff))]
    #[bw(map(|version| version | 0x8000))]
    pub version: u16,

    /// The type of the frame, either [`FrameType::Video`], [`FrameType::Audio`] or [`FrameType::Text`].
    pub frame_type: FrameType,

    /// Size of the header in the data segment.
    pub header_size: u32,

    /// Size of the payload, after the header, in the data segment.
    pub payload_size: u32,

    /// The payload of the frame.
    #[br(count = header_size + payload_size)]
    pub data: Vec<u8>,
}

impl Frame {
    pub fn unpack(mut self) -> Result<Msg> {
        let scrambler = Scrambler::detect(&self.frame_type, self.version);

        match self.frame_type {
            FrameType::Text => {
                scrambler.unscramble(&mut self.data[..], self.header_size + self.payload_size)
            }
            _ => scrambler.unscramble(
                &mut self.data[..self.header_size as usize],
                self.header_size + self.payload_size,
            ),
        }

        Ok(match self.frame_type {
            FrameType::Video => Msg::Video(Pack::from_frame(self)?),
            FrameType::Audio => Msg::Audio(Pack::from_frame(self)?),
            FrameType::Text => Msg::Text(Pack::from_frame(self)?),
        })
    }

    pub fn pack(msg: &Msg) -> Result<Self> {
        let (mut header, mut payload) = (Vec::new(), Vec::new());

        let frame_type = match msg {
            Msg::Video(inner) => {
                inner.header.write(&mut std::io::Cursor::new(&mut header))?;
                inner.data.write(&mut std::io::Cursor::new(&mut payload))?;

                FrameType::Video
            }
            Msg::Audio(inner) => {
                inner.header.write(&mut std::io::Cursor::new(&mut header))?;
                inner.data.write(&mut std::io::Cursor::new(&mut payload))?;

                FrameType::Audio
            }
            Msg::Text(inner) => {
                inner.header.write(&mut std::io::Cursor::new(&mut header))?;
                inner.data.write(&mut std::io::Cursor::new(&mut payload))?;

                FrameType::Text
            }
        };
        let version = frame_type.version();
        let header_size = header.len() as u32;
        let payload_size = payload.len() as u32;

        header.append(&mut payload);
        let mut data = header;

        let scrambler = Scrambler::detect(&frame_type, version);

        match frame_type {
            FrameType::Text => scrambler.scramble(&mut data[..], header_size + payload_size),
            _ => scrambler.scramble(
                &mut data[..header_size as usize],
                header_size + payload_size,
            ),
        }

        Ok(Frame {
            version,
            frame_type,
            header_size,
            payload_size,
            data,
        })
    }
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

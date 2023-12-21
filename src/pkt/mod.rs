use binrw::{BinRead, BinWrite};

use crate::Result;

mod scrambler;
pub use scrambler::Scrambler;

mod stream;
pub use stream::Stream;

use frame::{Block, Frame, FrameType};
pub mod frame;

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct Pkt {
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

impl Pkt {
    pub fn unpack(mut self) -> Result<frame::Frame> {
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
            FrameType::Video => Frame::Video(Block::from_pkt(self)?),
            FrameType::Audio => Frame::Audio(Block::from_pkt(self)?),
            FrameType::Text => Frame::Text(Block::from_pkt(self)?),
        })
    }

    pub fn pack(frame: &frame::Frame) -> Result<Self> {
        let (mut header, mut payload) = (Vec::new(), Vec::new());

        let frame_type = match frame {
            Frame::Video(inner) => {
                inner.header.write(&mut std::io::Cursor::new(&mut header))?;
                inner.data.write(&mut std::io::Cursor::new(&mut payload))?;

                FrameType::Video
            }
            Frame::Audio(inner) => {
                inner.header.write(&mut std::io::Cursor::new(&mut header))?;
                inner.data.write(&mut std::io::Cursor::new(&mut payload))?;

                FrameType::Audio
            }
            Frame::Text(inner) => {
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

        Ok(Pkt {
            version,
            frame_type,
            header_size,
            payload_size,
            data,
        })
    }
}

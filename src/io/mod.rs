use binrw::{BinRead, BinWrite};

mod scrambler;
pub use scrambler::Scrambler;

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct Frame {
    #[br(map(|version: u16| version & 0x7fff))] // FIXME: Handle when the MSB is `0`
    pub version: u16,

    pub frame_type: FrameType,

    pub header_size: u32,

    pub payload_len: u32,

    #[br(count = header_size + payload_len)]
    pub data: Vec<u8>,
}

impl Frame {
    pub fn unpack(mut self) -> Vec<u8> {
        let scrambler = Scrambler::identify(self.version, &self.frame_type);

        match self.frame_type {
            FrameType::Metadata => {
                scrambler.unscramble(&mut self.data[..], self.header_size + self.payload_len)
            }
            _ => scrambler.unscramble(
                &mut self.data[..self.header_size as usize],
                self.header_size + self.payload_len,
            ),
        }

        self.data
    }
}

#[derive(Debug, BinRead, BinWrite)]
#[brw(repr(u16))]
pub enum FrameType {
    Video = 0,
    Audio,
    Metadata,
}

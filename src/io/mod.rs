use binrw::{BinRead, BinWrite};

mod scrambler;
pub use scrambler::Scrambler;

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct Frame {
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

        scrambler.unscramble(&mut self.data[..], self.header_size + self.payload_len);

        tracing::warn!(
            "Unpacked: {:?} ({})",
            self.data,
            String::from_utf8_lossy(&self.data)
        );

        scrambler.unscramble(&mut self.data[..], self.header_size + self.payload_len);

        tracing::warn!(
            "Repacked: {:?} ({})",
            self.data,
            String::from_utf8_lossy(&self.data)
        );

        self.data
    }
}

#[derive(Debug, BinRead, BinWrite)]
#[brw(repr(u16))]
pub enum FrameType {
    Video = 0,
    Audio,
    Text,
}

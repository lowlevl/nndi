use binrw::{BinRead, BinWrite};

mod scrambler;
pub use scrambler::Scrambler;

mod stream;
pub use stream::Stream;

use frame::FrameType;
pub mod frame;

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct Packet {
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

use binrw::{BinRead, BinWrite};
use strum::AsRefStr;

pub type Block = super::Block<Spec, super::BytesEof>;

#[derive(Debug, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct Spec {
    pub fourcc: FourCCVideoType,
    pub width: u32,
    pub height: u32,
    pub fps_num: u32,
    pub fps_den: u32,
    pub aspect_ratio: f32,
    pub _1: u32,
    pub frame_format: FrameFormat,
    pub _2: u32,
    pub _3: u32,
    pub timecode: i64,
    pub _4: u32,
    pub _5: u32,
    pub metadata: binrw::NullString,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq, AsRefStr, BinRead, BinWrite)]
#[strum(serialize_all = "UPPERCASE")]
pub enum FourCCVideoType {
    #[brw(magic = b"SHQ2")]
    SHQ2,

    #[brw(magic = b"SHQ7")]
    SHQ7,
}

impl FourCCVideoType {
    pub fn to_code(&self) -> u32 {
        let bytes = self
            .as_ref()
            .as_bytes()
            .try_into()
            .expect("FourCC was not of 4 characters");

        u32::from_le_bytes(bytes)
    }
}

#[derive(Debug, PartialEq, BinRead, BinWrite)]
#[brw(repr = u32)]
pub enum FrameFormat {
    Interleaved = 0,
    Progressive,
    Field0,
    Field1,
}

use binrw::{BinRead, BinWrite};
use chrono::Utc;
use strum::AsRefStr;

pub type Block = super::Block<Spec, super::BytesEof>;

#[derive(Debug, Default, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct Spec {
    pub fourcc: FourCCVideoType,
    pub width: u32,
    pub height: u32,
    pub fps_num: u32,
    pub fps_den: u32,
    pub aspect_ratio: f32,
    pub _1: [u8; 4],
    pub frame_format: FrameFormat,
    pub _2: [u8; 4],
    pub _3: [u8; 4],
    pub _4: [u8; 4],
    pub _5: [u8; 4],
    pub timestamp: Timestamp,
    pub metadata: binrw::NullString,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Default, PartialEq, AsRefStr, BinRead, BinWrite)]
#[strum(serialize_all = "UPPERCASE")]
pub enum FourCCVideoType {
    #[brw(magic = b"SHQ2")]
    #[default]
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

#[derive(Debug, Default, PartialEq, BinRead, BinWrite)]
#[brw(repr = u32)]
pub enum FrameFormat {
    Interleaved = 0,

    #[default]
    Progressive,

    Field0,

    Field1,
}

#[derive(Debug, Default, PartialEq, Eq, BinRead, BinWrite)]
pub struct Timestamp(
    #[br(try_map = |timestamp: i64| chrono::DateTime::from_timestamp_micros(timestamp / 10).ok_or("Timestamp out-of-range"))]
    #[bw(map = |timestamp| timestamp.timestamp_micros() * 10)]
    chrono::DateTime<Utc>,
);

impl std::ops::Deref for Timestamp {
    type Target = chrono::DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::convert::From<chrono::DateTime<Utc>> for Timestamp {
    fn from(value: chrono::DateTime<Utc>) -> Self {
        Self(value)
    }
}

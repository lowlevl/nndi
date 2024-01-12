use binrw::{BinRead, BinWrite};
use ffmpeg::codec;
use strum::AsRefStr;

pub type Block = super::Block<Spec, super::BytesEof>;

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct Spec {
    pub fourcc: FourCCAudioType,
    pub samples: u32,
    pub num_channels: u32,
    pub sample_rate: u32,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, AsRefStr, BinRead, BinWrite)]
#[strum(serialize_all = "lowercase")]
pub enum FourCCAudioType {
    #[brw(magic = b"fowt")]
    FOWT,

    #[brw(magic = b"sowt")]
    SOWT,
}

impl FourCCAudioType {
    pub fn to_code(&self) -> u32 {
        let bytes = self
            .as_ref()
            .as_bytes()
            .try_into()
            .expect("FourCC was not of 4 characters");

        u32::from_le_bytes(bytes)
    }

    pub fn to_codec(&self) -> Option<ffmpeg::Codec> {
        codec::decoder::find(match self {
            FourCCAudioType::FOWT => codec::Id::PCM_S16LE,
            FourCCAudioType::SOWT => codec::Id::PCM_S16LE,
        })
    }
}

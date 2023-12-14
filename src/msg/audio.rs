use binrw::{BinRead, BinWrite};

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct Spec {
    pub fourcc: FourCCAudioType,
    pub samples: u32,
    pub num_channels: u32,
    pub sample_rate: u32,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, BinRead, BinWrite)]
pub enum FourCCAudioType {
    #[brw(magic = b"FLTP")]
    FLTP,
}

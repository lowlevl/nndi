use binrw::{BinRead, BinWrite};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, BinRead, BinWrite)]
pub enum FourCCAudioType {
    #[brw(magic = b"FLTP")]
    FLTP,
}

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct AudioSpec {
    pub fourcc: FourCCAudioType,
    pub samples: u32,
    pub num_channels: u32,
    pub sample_rate: u32,
}

use binrw::{BinRead, BinWrite};

pub type Block = super::Block<Spec, binrw::NullString>;

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
    #[brw(magic = b"fowt")]
    FOWT,

    #[brw(magic = b"sowt")]
    SOWT,
}

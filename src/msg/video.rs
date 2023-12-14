use binrw::{BinRead, BinWrite};

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct Spec {
    pub fourcc: FourCCVideoType,
    pub width: u32,
    pub height: u32,
    pub fps_num: u32,
    pub fps_den: u32,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, BinRead, BinWrite)]
pub enum FourCCVideoType {
    #[brw(magic = b"UYBY")]
    UYVY,

    #[brw(magic = b"UYVA")]
    UYVA,

    #[brw(magic = b"P216")]
    P216,

    #[brw(magic = b"PA16")]
    PA16,

    #[brw(magic = b"YV12")]
    YV12,

    #[brw(magic = b"I420")]
    I420,

    #[brw(magic = b"NV12")]
    NV12,

    #[brw(magic = b"BGRA")]
    BGRA,

    #[brw(magic = b"BGRX")]
    BGRX,

    #[brw(magic = b"RGBA")]
    RGBA,

    #[brw(magic = b"RGBX")]
    RGBX,
}

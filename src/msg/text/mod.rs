use binrw::{BinRead, BinWrite};

pub mod info;

#[derive(Debug, BinRead, BinWrite)]
#[brw(little)]
pub struct Text {
    header: [u8; 8],
    text: binrw::NullString,
}

impl Text {
    pub fn into_inner(self) -> Vec<u8> {
        self.text.into()
    }
}

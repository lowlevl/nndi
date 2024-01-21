use binrw::{
    meta::{ReadEndian, WriteEndian},
    BinRead, BinWrite,
};

use crate::Result;

#[derive(Debug, PartialEq)]
pub struct Block<H, D> {
    pub header: H,
    pub data: D,
}

impl<H: Default, D> Block<H, D> {
    pub fn data(data: impl Into<D>) -> Self {
        Self {
            header: Default::default(),
            data: data.into(),
        }
    }
}

impl<H, D> Block<H, D>
where
    H: for<'a> BinRead<Args<'a> = ()> + ReadEndian,
    D: for<'a> BinRead<Args<'a> = ()> + ReadEndian,
{
    pub fn from_raw(header: &[u8], data: &[u8]) -> Result<Self> {
        let header = BinRead::read(&mut std::io::Cursor::new(header))?;
        let data = BinRead::read(&mut std::io::Cursor::new(data))?;

        Ok(Self { header, data })
    }
}

impl<H, D> Block<H, D>
where
    H: for<'a> BinWrite<Args<'a> = ()> + WriteEndian,
    D: for<'a> BinWrite<Args<'a> = ()> + WriteEndian,
{
    pub fn to_raw(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let (mut header, mut data) = (Vec::new(), Vec::new());

        self.header.write(&mut std::io::Cursor::new(&mut header))?;
        self.data.write(&mut std::io::Cursor::new(&mut data))?;

        Ok((header, data))
    }
}

#[derive(Debug, PartialEq, BinRead, BinWrite)]
#[brw(little)]
pub struct BytesEof {
    #[br(parse_with = binrw::helpers::until_eof)]
    inner: Vec<u8>,
}

impl std::ops::Deref for BytesEof {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

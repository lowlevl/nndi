use binrw::{meta::ReadEndian, BinRead};

use crate::{io::Packet, Result};

#[derive(Debug)]
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
    pub fn from_pkt(pkt: Packet) -> Result<Self> {
        Ok(Self {
            header: BinRead::read(&mut std::io::Cursor::new(
                &pkt.data[..pkt.header_size as usize],
            ))?,
            data: BinRead::read(&mut std::io::Cursor::new(
                &pkt.data[pkt.header_size as usize..],
            ))?,
        })
    }
}

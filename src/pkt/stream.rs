use std::net::{TcpStream, ToSocketAddrs};

use binrw::{io::NoSeek, BinRead, BinWrite};

use super::{frame::Frame, Pkt};
use crate::Result;

pub struct Stream {
    stream: NoSeek<TcpStream>,
}

impl Stream {
    pub fn connect(addrs: impl ToSocketAddrs) -> Result<Self> {
        Ok(Self {
            stream: NoSeek::new(TcpStream::connect(addrs)?),
        })
    }

    pub fn send(&mut self, frame: &Frame) -> Result<()> {
        tracing::trace!(
            "Sending message to `{}` {frame:?}",
            self.stream.get_ref().peer_addr()?
        );

        let pkt = Pkt::pack(frame)?;
        pkt.write(&mut self.stream)?;

        Ok(())
    }

    pub fn recv(&mut self) -> Result<Frame> {
        let pkt = Pkt::read(&mut self.stream)?;
        let frame = pkt.unpack()?;

        tracing::trace!(
            "Receiving message from `{}` {frame:?}",
            self.stream.get_ref().peer_addr()?
        );

        Ok(frame)
    }
}

impl std::ops::Deref for Stream {
    type Target = TcpStream;

    fn deref(&self) -> &Self::Target {
        self.stream.get_ref()
    }
}

impl std::convert::From<TcpStream> for Stream {
    fn from(value: TcpStream) -> Stream {
        Self {
            stream: NoSeek::new(value),
        }
    }
}

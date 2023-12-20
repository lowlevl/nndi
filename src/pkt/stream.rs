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

    pub fn send(&mut self, msg: &Frame) -> Result<()> {
        tracing::trace!(
            "Sending message to `{}` {msg:?}",
            self.stream.get_ref().peer_addr()?
        );

        let frame = Pkt::pack(msg)?;
        frame.write(&mut self.stream)?;

        Ok(())
    }

    pub fn recv(&mut self) -> Result<Frame> {
        let frame = Pkt::read(&mut self.stream)?;
        let msg = frame.unpack()?;

        tracing::trace!(
            "Receiving message from `{}` {msg:?}",
            self.stream.get_ref().peer_addr()?
        );

        Ok(msg)
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

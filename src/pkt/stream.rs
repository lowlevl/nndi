use std::net::{TcpStream, ToSocketAddrs};

use binrw::{io::NoSeek, BinRead, BinWrite};

use super::{msg::Msg, Pkt};
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

    pub fn send(&mut self, msg: &Msg) -> Result<()> {
        tracing::debug!(
            "Sending message to `{}` {msg:?}",
            self.stream.get_ref().peer_addr()?
        );

        let frame = Pkt::pack(msg)?;
        tracing::trace!(
            "Sending frame to `{}` {frame:?}",
            self.stream.get_ref().peer_addr()?
        );

        frame.write(&mut self.stream)?;

        Ok(())
    }

    pub fn recv(&mut self) -> Result<Msg> {
        let frame = Pkt::read(&mut self.stream)?;
        tracing::trace!(
            "Receiving frame from `{}` {frame:?}",
            self.stream.get_ref().peer_addr()?
        );

        let msg = frame.unpack()?;
        tracing::debug!(
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

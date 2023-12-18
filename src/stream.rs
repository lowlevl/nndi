use std::net::{TcpStream, ToSocketAddrs};

use binrw::{io::NoSeek, BinRead, BinWrite};

use crate::{frame::Frame, msg::Msg, Result};

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
        Frame::pack(msg)?.write(&mut self.stream)?;

        tracing::trace!("Sent message: {msg:?}");

        Ok(())
    }

    pub fn recv(&mut self) -> Result<Msg> {
        let msg = Frame::read(&mut self.stream)?.unpack()?;

        tracing::trace!("Received message: {msg:?}");

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

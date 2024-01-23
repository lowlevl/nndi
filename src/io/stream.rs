use tokio::{
    io::{AsyncWriteExt, BufStream},
    net::TcpStream,
};

use super::{
    frame::{text::Metadata, Frame},
    Packet,
};
use crate::Result;

#[derive(Debug)]
pub struct Stream {
    stream: BufStream<TcpStream>,
}

impl Stream {
    pub async fn recv(&mut self) -> Result<Frame> {
        Packet::read(&mut self.stream).await?.into_frame()
    }

    pub async fn send(&mut self, frame: &Frame) -> Result {
        Packet::from_frame(frame).write(&mut self.stream).await?;

        Ok(self.stream.flush().await?)
    }

    pub async fn readable(&self) -> Result {
        Ok(self.stream.get_ref().readable().await?)
    }

    pub async fn writable(&self) -> Result {
        Ok(self.stream.get_ref().writable().await?)
    }

    /// Retrieve the next message and convert it to [`Metadata`] if possible, discarding otherwise.
    pub async fn metadata(&mut self) -> Result<Option<Metadata>> {
        match self.recv().await? {
            Frame::Text(block) => Ok(Some(Metadata::from_block(&block)?)),
            _ => Ok(None),
        }
    }
}

impl std::convert::From<TcpStream> for Stream {
    fn from(stream: TcpStream) -> Self {
        Self {
            stream: BufStream::new(stream),
        }
    }
}

impl std::ops::Deref for Stream {
    type Target = TcpStream;

    fn deref(&self) -> &Self::Target {
        self.stream.get_ref()
    }
}

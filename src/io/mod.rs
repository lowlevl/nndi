use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{Error, Result};

mod scrambler;
pub use scrambler::Scrambler;

mod stream;
pub use stream::Stream;

pub mod frame;

use frame::{Frame, FrameKind};

#[derive(Debug, Clone, PartialEq)]
pub struct Packet {
    version: u16,
    kind: FrameKind,
    header_size: usize,
    data: Vec<u8>,
}

impl Packet {
    pub async fn read(mut stream: impl tokio::io::AsyncRead + Unpin) -> Result<Self> {
        let version = stream.read_u16_le().await? & 0x7fff;
        let kind = FrameKind::from_repr(stream.read_u16_le().await?).ok_or(Error::UnknownKind)?;
        let header_size = stream.read_u32_le().await? as usize;
        let payload_size = stream.read_u32_le().await?;

        let mut data = vec![0; header_size + payload_size as usize];
        stream.read_exact(&mut data[..]).await?;

        Ok(Self {
            version,
            kind,
            header_size,
            data,
        })
    }

    pub async fn write(&self, mut stream: impl tokio::io::AsyncWrite + Unpin) -> Result<()> {
        stream.write_u16_le(self.version | 0x8000).await?;
        stream.write_u16_le(self.kind as u16).await?;
        stream.write_u32_le(self.header_size as u32).await?;
        stream
            .write_u32_le((self.data.len() - self.header_size) as u32)
            .await?;

        stream.write_all(&self.data).await?;

        Ok(())
    }

    pub fn into_frame(mut self) -> Result<Frame> {
        let scrambler = Scrambler::new(&self.kind, self.version);
        let seed = self.data.len() as u32;

        match self.kind {
            FrameKind::Text => scrambler.unscramble(&mut self.data[..], seed),
            _ => scrambler.unscramble(&mut self.data[..self.header_size], seed),
        }

        Frame::from_parts(
            self.kind,
            &self.data[..self.header_size],
            &self.data[self.header_size..],
        )
    }

    pub fn from_frame(frame: &Frame) -> Result<Self> {
        let (kind, mut header, mut data) = frame.to_parts()?;

        let version = kind.version();
        let header_size = header.len();

        header.append(&mut data);
        let mut data = header;

        let scrambler = Scrambler::new(&kind, version);
        let seed = data.len() as u32;

        match kind {
            FrameKind::Text => scrambler.scramble(&mut data[..], seed),
            _ => scrambler.scramble(&mut data[..header_size], seed),
        }

        Ok(Self {
            version,
            kind,
            header_size,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use self::frame::Block;

    use super::*;

    #[tokio::test]
    async fn it_converts_frames_and_messages() -> Result<(), Box<dyn std::error::Error>> {
        let mut bytes = Vec::new();

        let frame = Frame::Text(Block::data("hello world !"));
        let message = Packet::from_frame(&frame)?;

        message.write(&mut bytes).await?;

        let message2 = Packet::read(&mut std::io::Cursor::new(&bytes)).await?;
        assert_eq!(message, message2);

        let frame2 = message2.into_frame()?;
        assert_eq!(frame, frame2);

        Ok(())
    }
}

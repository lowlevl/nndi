use std::net::{TcpStream, ToSocketAddrs};

use binrw::{io::NoSeek, BinRead, BinWrite};

use super::{
    frame::{Block, Frame, FrameType},
    Packet, Scrambler,
};
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
            "Sending frame to `{}` {frame:?}",
            self.stream.get_ref().peer_addr()?
        );

        let (mut header, mut payload) = (Vec::new(), Vec::new());

        let frame_type = match frame {
            Frame::Video(inner) => {
                inner.header.write(&mut std::io::Cursor::new(&mut header))?;
                inner.data.write(&mut std::io::Cursor::new(&mut payload))?;

                FrameType::Video
            }
            Frame::Audio(inner) => {
                inner.header.write(&mut std::io::Cursor::new(&mut header))?;
                inner.data.write(&mut std::io::Cursor::new(&mut payload))?;

                FrameType::Audio
            }
            Frame::Text(inner) => {
                inner.header.write(&mut std::io::Cursor::new(&mut header))?;
                inner.data.write(&mut std::io::Cursor::new(&mut payload))?;

                FrameType::Text
            }
        };
        let version = frame_type.version();
        let header_size = header.len() as u32;
        let payload_size = payload.len() as u32;

        header.append(&mut payload);
        let mut data = header;

        let scrambler = Scrambler::detect(&frame_type, version);

        match frame_type {
            FrameType::Text => scrambler.scramble(&mut data[..], header_size + payload_size),
            _ => scrambler.scramble(
                &mut data[..header_size as usize],
                header_size + payload_size,
            ),
        }

        let packet = Packet {
            version,
            frame_type,
            header_size,
            payload_size,
            data,
        };
        packet.write(&mut self.stream)?;

        Ok(())
    }

    pub fn recv(&mut self) -> Result<Frame> {
        let mut packet = Packet::read(&mut self.stream)?;

        let scrambler = Scrambler::detect(&packet.frame_type, packet.version);

        match packet.frame_type {
            FrameType::Text => scrambler.unscramble(
                &mut packet.data[..],
                packet.header_size + packet.payload_size,
            ),
            _ => scrambler.unscramble(
                &mut packet.data[..packet.header_size as usize],
                packet.header_size + packet.payload_size,
            ),
        }

        let frame = match packet.frame_type {
            FrameType::Video => Frame::Video(Block::from_pkt(packet)?),
            FrameType::Audio => Frame::Audio(Block::from_pkt(packet)?),
            FrameType::Text => Frame::Text(Block::from_pkt(packet)?),
        };

        tracing::trace!(
            "Receiving frame from `{}` {frame:?}",
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

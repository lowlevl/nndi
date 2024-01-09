use std::{
    io::Write,
    net::{TcpStream, ToSocketAddrs},
};

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
        let stream = NoSeek::new(TcpStream::connect(addrs)?);

        Ok(Self { stream })
    }

    pub fn send(&mut self, frame: &Frame) -> Result<()> {
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

        let seed = header_size + payload_size;
        match frame_type {
            FrameType::Text => scrambler.scramble(&mut data[..], seed),
            _ => scrambler.scramble(&mut data[..header_size as usize], seed),
        }

        let packet = Packet {
            version,
            frame_type,
            header_size,
            payload_size,
            data,
        };
        packet.write(&mut self.stream)?;
        self.stream.get_mut().flush()?;

        tracing::trace!(
            "Sent packet to `{}`: version = {}, type = {:?}, len = {}",
            self.stream.get_ref().peer_addr()?,
            packet.version,
            packet.frame_type,
            packet.header_size + packet.payload_size
        );

        Ok(())
    }

    pub fn recv(&mut self) -> Result<Frame> {
        let mut packet = Packet::read(&mut self.stream)?;

        tracing::trace!(
            "Recevied packet to `{}`: version = {}, type = {:?}, len = {}",
            self.stream.get_ref().peer_addr()?,
            packet.version,
            packet.frame_type,
            packet.header_size + packet.payload_size
        );

        let scrambler = Scrambler::detect(&packet.frame_type, packet.version);

        let seed = packet.header_size + packet.payload_size;
        match packet.frame_type {
            FrameType::Text => scrambler.unscramble(&mut packet.data[..], seed),
            _ => scrambler.unscramble(&mut packet.data[..packet.header_size as usize], seed),
        }

        let frame = match packet.frame_type {
            FrameType::Video => Frame::Video(Block::from_pkt(packet)?),
            FrameType::Audio => Frame::Audio(Block::from_pkt(packet)?),
            FrameType::Text => Frame::Text(Block::from_pkt(packet)?),
        };

        Ok(frame)
    }
}

impl std::ops::Deref for Stream {
    type Target = TcpStream;

    fn deref(&self) -> &Self::Target {
        self.stream.get_ref()
    }
}

impl std::ops::DerefMut for Stream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.stream.get_mut()
    }
}

impl std::convert::From<TcpStream> for Stream {
    fn from(value: TcpStream) -> Stream {
        Self {
            stream: NoSeek::new(value),
        }
    }
}

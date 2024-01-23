use std::net::SocketAddr;

use crate::{
    io::{
        frame::{
            text::{self, Metadata},
            Frame,
        },
        Stream,
    },
    Result,
};

#[cfg(doc)]
use super::Source;

/// A _peer_ currently connected to a [`Source`], with all of it's protocol parameters.
#[derive(Debug, Clone)]
pub struct Peer {
    pub addr: SocketAddr,
    pub version: text::Version,
    pub identify: text::Identify,
    pub streams: text::EnabledStreams,
    pub quality: text::VideoQuality,
    pub tally: text::Tally,
}

impl Peer {
    async fn greet(stream: &mut Stream) -> Result {
        stream.send(&Frame::version()).await?;
        stream.send(&Frame::identify("?")).await?;

        Ok(())
    }

    pub(super) async fn handshake(addr: SocketAddr, stream: &mut Stream) -> Result<Self> {
        Self::greet(stream).await?;

        let mut version = None;
        let mut identify = None;
        let mut streams = None;
        let mut quality = None;
        let mut tally = Default::default();

        loop {
            match stream.metadata().await? {
                Some(Metadata::Version(value)) => version = Some(value),
                Some(Metadata::Identify(value)) => identify = Some(value),
                Some(Metadata::EnabledStreams(value)) => streams = Some(value),
                Some(Metadata::Video(value)) => quality = Some(value.quality),
                Some(Metadata::Tally(value)) => tally = value,
                _ => continue,
            }

            if version.is_some() && identify.is_some() && streams.is_some() && quality.is_some() {
                #[allow(clippy::unwrap_used)] // Checked if the value is Some(T) just before
                let peer = Self {
                    addr,
                    version: version.take().unwrap(),
                    identify: identify.take().unwrap(),
                    streams: streams.take().unwrap(),
                    quality: quality.take().unwrap(),
                    tally,
                };

                tracing::debug!(
                    "New peer connected from `{}`: {peer:?}",
                    stream.peer_addr()?
                );

                break Ok(peer);
            }
        }
    }
}

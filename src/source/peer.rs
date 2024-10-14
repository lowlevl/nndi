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

use super::Config;
#[cfg(doc)]
use super::Source;

/// A _peer_ currently connected to a [`Source`], with all of it's protocol parameters.
#[derive(Debug, Clone)]
pub struct Peer {
    /// The software _version_ of the peer.
    pub version: text::Version,

    /// The _name_ of the peer.
    pub identify: text::Identify,

    /// The _enabled streams_ of the peer.
    pub streams: text::EnabledStreams,

    /// The _stream quality_ of the peer.
    pub quality: text::VideoQuality,

    /// The _tally_ of the peer.
    pub tally: text::Tally,
}

impl Peer {
    async fn greet(stream: &mut Stream, config: &Config) -> Result {
        stream.send(&Frame::version()).await?;
        stream.send(&Frame::identify(&config.name)).await?;

        Ok(())
    }

    pub(super) async fn handshake(stream: &mut Stream, config: &Config) -> Result<Self> {
        Self::greet(stream, config).await?;

        let mut version = None;
        let mut identify = None;
        let mut streams = None;
        let mut quality = Default::default();
        let mut tally = Default::default();

        loop {
            match stream.metadata().await? {
                Some(Metadata::Version(value)) => version = Some(value),
                Some(Metadata::Identify(value)) => identify = Some(value),
                Some(Metadata::EnabledStreams(value)) => streams = Some(value),
                Some(Metadata::Video(value)) => quality = value.quality,
                Some(Metadata::Tally(value)) => tally = value,
                _ => continue,
            }

            if version.is_some() && identify.is_some() && streams.is_some() {
                #[allow(clippy::unwrap_used)] // Checked if the value is Some(T) just before
                let peer = Self {
                    version: version.take().unwrap(),
                    identify: identify.take().unwrap(),
                    streams: streams.take().unwrap(),
                    quality,
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

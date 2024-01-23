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

use super::Config;

#[cfg(doc)]
use super::Sink;

/// A _peer_ currently connected to a [`Sink`], with all of it's protocol parameters.
#[derive(Debug, Clone)]
pub struct Peer {
    pub addr: SocketAddr,
    pub version: text::Version,
    pub identify: text::Identify,
}

impl Peer {
    async fn greet(stream: &mut Stream, config: &Config) -> Result {
        stream.send(&Frame::version()).await?;
        stream
            .send(&Frame::identify(
                config.name.as_deref().unwrap_or("generic sink"),
            ))
            .await?;
        stream
            .send(&Frame::video_meta(config.video_quality.clone()))
            .await?;
        stream
            .send(&Frame::enabled_streams(
                config.video_queue != 0,
                config.audio_queue != 0,
            ))
            .await?;

        Ok(())
    }

    pub(super) async fn handshake(stream: &mut Stream, config: &Config) -> Result<Self> {
        Self::greet(stream, config).await?;

        let mut version = None;
        let mut identify = None;

        loop {
            match stream.metadata().await? {
                Some(Metadata::Version(value)) => version = Some(value),
                Some(Metadata::Identify(value)) => identify = Some(value),
                _ => continue,
            }

            if version.is_some() && identify.is_some() {
                #[allow(clippy::unwrap_used)] // Checked if the value is Some(T) just before
                let peer = Self {
                    addr: stream.peer_addr()?,
                    version: version.take().unwrap(),
                    identify: identify.take().unwrap(),
                };

                tracing::debug!(
                    "Connected to network source `{}`: {peer:?}",
                    stream.peer_addr()?
                );

                break Ok(peer);
            }
        }
    }
}

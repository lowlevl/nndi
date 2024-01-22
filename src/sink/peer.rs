use crate::{
    io::{
        frame::text::{self, Metadata},
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
    pub version: text::Version,
    pub identify: text::Identify,
}

impl Peer {
    async fn greet(stream: &mut Stream, config: &Config) -> Result<()> {
        stream
            .send(
                &Metadata::Version(text::Version {
                    video: 5,
                    audio: 4,
                    text: 3,
                    sdk: crate::SDK_VERSION.into(),
                    platform: crate::SDK_PLATFORM.into(),
                })
                .to_block()?
                .into(),
            )
            .await?;

        stream
            .send(
                &Metadata::Identify(text::Identify {
                    name: crate::name(config.name.as_deref().unwrap_or("receiver")),
                })
                .to_block()?
                .into(),
            )
            .await?;

        stream
            .send(
                &Metadata::Video(text::Video {
                    quality: config.video_quality.clone(),
                })
                .to_block()?
                .into(),
            )
            .await?;

        stream
            .send(
                &Metadata::EnabledStreams(text::EnabledStreams {
                    video: config.video_queue != 0,
                    audio: config.audio_queue != 0,
                    text: true,
                    shq_skip_block: false,
                    shq_short_dc: false,
                })
                .to_block()?
                .into(),
            )
            .await?;

        Ok(())
    }

    pub(super) async fn handshake(stream: &mut Stream, config: &Config) -> Result<Self> {
        Self::greet(stream, config).await?;

        let mut version = None;
        let mut identify = None;

        loop {
            match stream.metadata().await? {
                Metadata::Version(value) => version = Some(value),
                Metadata::Identify(value) => identify = Some(value),
                _ => continue,
            }

            if version.is_some() && identify.is_some() {
                #[allow(clippy::unwrap_used)] // Checked if the value is Some(T) just before
                let peer = Self {
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

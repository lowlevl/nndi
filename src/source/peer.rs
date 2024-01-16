use std::time::{Duration, Instant};

use crate::{
    io::{
        frame::text::{self, Metadata},
        Stream,
    },
    Error, Result,
};

#[derive(Debug)]
pub struct Peer {
    pub version: text::Version,
    pub identify: text::Identify,
    pub streams: text::EnabledStreams,
    pub quality: text::VideoQuality,
    pub tally: text::Tally,
}

impl Peer {
    fn greet(stream: &mut Stream) -> Result<()> {
        stream.send(
            Metadata::Version(text::Version {
                video: 5,
                audio: 4,
                text: 3,
                sdk: crate::SDK_VERSION.into(),
                platform: crate::SDK_PLATFORM.into(),
            })
            .to_block()?,
        )?;

        stream.send(
            Metadata::Identify(text::Identify {
                name: crate::name("receiver"),
            })
            .to_block()?,
        )?;

        Ok(())
    }

    pub fn handshake(stream: &mut Stream, timeout: Duration) -> Result<Self> {
        let mut version = None;
        let mut identify = None;
        let mut streams = None;
        let mut quality = None;
        let mut tally = Default::default();

        let now = Instant::now();
        loop {
            if now.elapsed() >= timeout {
                break Err(Error::Timeout);
            }

            match stream.metadata()? {
                Metadata::Version(value) => version = Some(value),
                Metadata::Identify(value) => identify = Some(value),
                Metadata::EnabledStreams(value) => streams = Some(value),
                Metadata::Video(value) => quality = Some(value.quality),
                Metadata::Tally(value) => tally = value,
                _ => continue,
            }

            if version.is_some() && identify.is_some() && streams.is_some() && quality.is_some() {
                #[allow(clippy::unwrap_used)] // Checked if the value is Some(T) just before
                let peer = Self {
                    version: version.take().unwrap(),
                    identify: identify.take().unwrap(),
                    streams: streams.take().unwrap(),
                    quality: quality.take().unwrap(),
                    tally,
                };
                Self::greet(stream)?;

                tracing::debug!(
                    "New peer connected from `{}`: {peer:?}",
                    stream.peer_addr()?
                );

                break Ok(peer);
            }
        }
    }

    pub fn tick(&mut self, stream: &mut Stream) -> Result<()> {
        match stream.metadata()? {
            Metadata::Tally(tally) => {
                self.tally = tally.clone();

                stream.send(Metadata::TallyEcho(tally).to_block()?)?;
            }
            meta => tracing::debug!("Received unhandled metadata from peer: {meta:?}"),
        }

        Ok(())
    }
}

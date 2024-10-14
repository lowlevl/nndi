//! Everything related to NDI [`Sink`]s, to receive video.

use std::net::SocketAddr;

use ffmpeg::codec;
use futures::TryFutureExt;
use itertools::Itertools;
use mdns_sd::ServiceInfo;
use tokio::net::TcpStream;

use crate::{
    io::{
        frame::{audio, text::Metadata, video, Frame},
        Stream,
    },
    Error, Result,
};

mod config;
pub use config::Config;

mod peer;
pub use peer::Peer;

/// A _video_ and _audio_ sink, that can receive data from a source.
#[derive(Debug, Clone)]
pub struct Sink {
    peer: Peer,

    video: flume::Receiver<video::Block>,
    audio: flume::Receiver<audio::Block>,
}

impl Sink {
    /// Create a new [`Sink`] based on the provided `config` and `service` entry.
    pub async fn new(service: &ServiceInfo, config: Config<'_>) -> Result<Self> {
        let addresses = service
            .get_addresses()
            .iter()
            .map(|addr| SocketAddr::new(*addr, service.get_port()))
            .collect::<Vec<_>>();
        let mut stream: Stream = TcpStream::connect(addresses.as_slice()).await?.into();

        let peer = tokio::time::timeout(
            crate::HANDSHAKE_TIMEOUT,
            Peer::handshake(&mut stream, &config),
        )
        .await??;

        let (videotx, video) = flume::bounded(config.video_queue);
        let (audiotx, audio) = flume::bounded(config.audio_queue);
        tokio::spawn(
            Self::task(stream, videotx, audiotx)
                .inspect_err(|err| tracing::error!("Fatal error in `Sink::task`: {err}")),
        );

        Ok(Self { peer, video, audio })
    }

    /// Access the source [`Peer`] definition.
    pub fn peer(&self) -> &Peer {
        &self.peer
    }

    async fn task(
        mut stream: Stream,
        video: flume::Sender<video::Block>,
        audio: flume::Sender<audio::Block>,
    ) -> Result {
        loop {
            if video.is_disconnected() && audio.is_disconnected() {
                tracing::trace!("All receivers dropped, disconnecting from peer");

                break Ok(());
            }

            match stream.recv().await? {
                Frame::Video(block) => {
                    if let Err(err) = video.try_send(block) {
                        tracing::debug!("A video block was dropped: {err}");
                    }
                }
                Frame::Audio(block) => {
                    if let Err(err) = audio.try_send(block) {
                        tracing::debug!("An audio block was dropped: {err}");
                    }
                }
                Frame::Text(block) => {
                    let Ok(info) = Metadata::from_block(&block) else {
                        tracing::warn!(
                            "Unhandled information: {}",
                            String::from_utf8_lossy(&block.data)
                        );

                        continue;
                    };

                    tracing::warn!("Received information: {info:?}");
                }
            }
        }
    }

    /// Iterate over incoming [`video::Block`]s.
    fn video_blocks(&self) -> impl Iterator<Item = Result<video::Block, flume::RecvError>> + '_ {
        std::iter::from_fn(move || Some(self.video.recv()))
    }

    /// Iterate over decoded [`ffmpeg::frame::Video`] from incoming blocks.
    pub fn video_frames(&self) -> impl Iterator<Item = Result<ffmpeg::frame::Video>> + '_ {
        self.video_blocks()
            .map(|block| {
                let block = block.map_err(|_| Error::ClosedChannel)?;

                tracing::trace!("<- new block {block:?} from `{}`", self.peer.identify.name);

                let mut context = codec::Context::new();
                // SAFETY: The pointer is allocated on the line before,
                // and is guaranteed to be exclusive with `as_mut_ptr`.
                unsafe {
                    (*context.as_mut_ptr()).codec_tag = block.header.fourcc.to_code();
                    (*context.as_mut_ptr()).width = block.header.width as i32;
                    (*context.as_mut_ptr()).height = block.header.height as i32;
                    (*context.as_mut_ptr()).framerate = ffmpeg::ffi::AVRational {
                        num: block.header.fps_num as i32,
                        den: block.header.fps_den as i32,
                    };
                }

                let mut decoder = context
                    .decoder()
                    .open_as(codec::decoder::find(codec::Id::SPEEDHQ))?
                    .video()?;

                decoder.send_packet(&codec::packet::Packet::borrow(&block.data))?;
                decoder.send_eof()?;

                Ok::<_, crate::Error>(std::iter::from_fn(move || {
                    let mut frame = ffmpeg::frame::Video::empty();
                    decoder.receive_frame(&mut frame).is_ok().then_some(frame)
                }))
            })
            .flatten_ok()
    }

    /// Iterate over incoming [`audio::Block`]s.
    fn audio_blocks(&self) -> impl Iterator<Item = Result<audio::Block, flume::RecvError>> + '_ {
        std::iter::from_fn(move || Some(self.audio.recv()))
    }

    /// Iterate over decoded [`ffmpeg::frame::Audio`] from incoming blocks.
    pub fn audio_frames(&self) -> impl Iterator<Item = Result<ffmpeg::frame::Audio>> + '_ {
        self.audio_blocks()
            .map(|block| {
                let block = block.map_err(|_| Error::ClosedChannel)?;

                let mut context = codec::Context::new();
                // SAFETY: The pointer is allocated on the line before,
                // and is guaranteed to be exclusive with `as_mut_ptr`.
                unsafe {
                    (*context.as_mut_ptr()).codec_tag = block.header.fourcc.to_code();
                    (*context.as_mut_ptr()).channels = block.header.num_channels as i32;
                    (*context.as_mut_ptr()).sample_rate = block.header.sample_rate as i32;
                }

                let mut decoder = context
                    .decoder()
                    .open_as(block.header.fourcc.to_codec())?
                    .audio()?;

                decoder.send_packet(&codec::packet::Packet::borrow(&block.data))?;
                decoder.send_eof()?;

                Ok::<_, crate::Error>(std::iter::from_fn(move || {
                    let mut frame = ffmpeg::frame::Audio::empty();
                    decoder.receive_frame(&mut frame).is_ok().then_some(frame)
                }))
            })
            .flatten_ok()
    }
}

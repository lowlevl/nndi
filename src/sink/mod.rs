use std::{net::SocketAddr, thread};

use ffmpeg::codec;
use itertools::Itertools;
use mdns_sd::ServiceInfo;

use crate::{
    io::{
        frame::{
            audio,
            text::{self, Metadata},
            video, Frame,
        },
        Stream,
    },
    Result,
};

mod config;
pub use config::Config;

/// A _video_ and _audio_ sink, that can receive data from a source.
#[derive(Debug, Clone)]
pub struct Sink {
    video: flume::Receiver<video::Block>,
    audio: flume::Receiver<audio::Block>,
}

impl Sink {
    pub fn new(service: &ServiceInfo, config: Config) -> Result<Self> {
        let port = service.get_port();
        let mut stream = Stream::connect(
            &*service
                .get_addresses()
                .iter()
                .map(|addr| SocketAddr::new(*addr, port))
                .collect::<Vec<_>>(),
        )?;

        tracing::debug!(
            "Connected to network source `{}@{}`",
            service.get_fullname(),
            stream.peer_addr()?
        );

        Self::identify(&mut stream, &config)?;
        let (video, audio) = Self::task(stream, &config);

        Ok(Self { video, audio })
    }

    fn identify(stream: &mut Stream, config: &Config) -> Result<()> {
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
                name: crate::name(config.name.as_deref().unwrap_or("receiver")),
            })
            .to_block()?,
        )?;

        stream.send(
            Metadata::Video(text::Video {
                quality: config.video_quality.clone(),
            })
            .to_block()?,
        )?;

        stream.send(
            Metadata::EnabledStreams(text::EnabledStreams {
                video: config.video_queue != 0,
                audio: config.audio_queue != 0,
                text: true,
                shq_skip_block: false,
                shq_short_dc: false,
            })
            .to_block()?,
        )?;

        Ok(())
    }

    fn task(
        mut stream: Stream,
        config: &Config,
    ) -> (flume::Receiver<video::Block>, flume::Receiver<audio::Block>) {
        let (video, videorx) = flume::bounded(config.video_queue);
        let (audio, audiorx) = flume::bounded(config.audio_queue);

        let mut task = move || {
            loop {
                if video.is_disconnected() && audio.is_disconnected() {
                    tracing::trace!("All receivers dropped, disconnecting from peer");

                    break;
                }

                match stream.recv()? {
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

            Ok::<_, crate::Error>(())
        };

        thread::spawn(move || {
            if let Err(err) = task() {
                tracing::error!("Fatal error in the `Sink::task` thread: {err}");
            }
        });

        (videorx, audiorx)
    }

    /// Iterate over incoming [`video::Block`]s.
    fn video_blocks(&self) -> impl Iterator<Item = Result<video::Block, flume::RecvError>> + '_ {
        std::iter::from_fn(move || Some(self.video.recv()))
    }

    /// Iterate over decoded [`ffmpeg::frame::Video`] from incoming blocks.
    pub fn video_frames(&self) -> impl Iterator<Item = Result<ffmpeg::frame::Video>> + '_ {
        self.video_blocks()
            .map(|block| {
                let block = block?;

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
                let block = block?;

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

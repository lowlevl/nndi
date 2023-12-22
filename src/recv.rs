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

#[derive(Debug, Clone)]
pub struct Recv {
    video: flume::Receiver<video::Block>,
    audio: flume::Receiver<audio::Block>,
}

impl Recv {
    pub fn new(service: &ServiceInfo, queue: usize) -> Result<Self> {
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

        Self::identify(&mut stream)?;

        let (videotx, video) = flume::bounded(queue);
        let (audiotx, audio) = flume::bounded(queue);
        Self::task(stream, videotx, audiotx);

        Ok(Self { video, audio })
    }

    fn identify(stream: &mut Stream) -> Result<()> {
        stream.send(&Frame::Text(
            Metadata::Version(text::Version {
                video: 5,
                audio: 4,
                text: 3,
                sdk: crate::SDK_VERSION.into(),
                platform: crate::SDK_PLATFORM.into(),
            })
            .to_block()?,
        ))?;

        stream.send(&Frame::Text(
            Metadata::Identify(text::Identify {
                name: crate::name("receiver"),
            })
            .to_block()?,
        ))?;

        stream.send(&Frame::Text(
            Metadata::Video(text::Video {
                quality: text::VideoQuality::High,
            })
            .to_block()?,
        ))?;

        stream.send(&Frame::Text(
            Metadata::EnabledStreams(text::EnabledStreams {
                video: true,
                audio: true,
                text: true,
                shq_skip_block: true,
                shq_short_dc: true,
            })
            .to_block()?,
        ))?;

        Ok(())
    }

    fn task(
        mut stream: Stream,
        video: flume::Sender<video::Block>,
        audio: flume::Sender<audio::Block>,
    ) {
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
                    Frame::Text(_) => {}
                }
            }

            Ok::<_, crate::Error>(())
        };

        thread::spawn(move || {
            if let Err(err) = task() {
                tracing::error!("Fatal error in the `Recv::task` thread: {err}");
            }
        });
    }

    /// Pop the next [`video::Block`] from the queue, if present.
    pub fn video(&self) -> Result<video::Block, flume::TryRecvError> {
        self.video.try_recv()
    }

    /// Iterate forever over the [`video::Block`] from the queue.
    pub fn video_blocks(
        &self,
    ) -> impl Iterator<Item = Result<video::Block, flume::RecvError>> + '_ {
        std::iter::from_fn(move || Some(self.video.recv()))
    }

    /// Iterate forever over decoded [`ffmpeg::frame::Video`] .
    pub fn video_frames(&self) -> impl Iterator<Item = Result<ffmpeg::util::frame::Video>> + '_ {
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

    /// Pop the next [`audio::Block`] from the queue, if present.
    pub fn audio(&self) -> Result<audio::Block, flume::TryRecvError> {
        self.audio.try_recv()
    }

    /// Iterate forever over the [`audio::Block`] from the queue.
    pub fn iter_audio(&self) -> impl Iterator<Item = Result<audio::Block, flume::RecvError>> + '_ {
        std::iter::from_fn(move || Some(self.audio.recv()))
    }
}

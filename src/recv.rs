use std::{net::SocketAddr, thread};

use mdns_sd::ServiceInfo;

use crate::{
    msg::{
        audio,
        metadata::{self, Metadata},
        video, Msg,
    },
    stream::Stream,
    Result,
};

pub struct Recv {
    video: flume::Receiver<video::Pack>,
    audio: flume::Receiver<audio::Pack>,
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

        stream.send(&Msg::Text(
            Metadata::Version(metadata::Version {
                video: 5,
                audio: 4,
                text: 3,
                sdk: crate::SDK_VERSION.into(),
                platform: crate::SDK_PLATFORM.into(),
            })
            .to_pack()?,
        ))?;

        stream.send(&Msg::Text(
            Metadata::Identify(metadata::Identify {
                name: crate::name("receiver")?,
            })
            .to_pack()?,
        ))?;

        stream.send(&Msg::Text(
            Metadata::Video(metadata::Video {
                quality: metadata::VideoQuality::High,
            })
            .to_pack()?,
        ))?;

        stream.send(&Msg::Text(
            Metadata::EnabledStreams(metadata::EnabledStreams {
                video: true,
                audio: true,
                text: true,
                shq_skip_block: true,
                shq_short_dc: true,
            })
            .to_pack()?,
        ))?;

        let (videotx, video) = flume::bounded(queue);
        let (audiotx, audio) = flume::bounded(queue);
        Self::task(stream, videotx, audiotx);

        Ok(Self { video, audio })
    }

    fn task(
        mut stream: Stream,
        video: flume::Sender<video::Pack>,
        audio: flume::Sender<audio::Pack>,
    ) {
        let mut task = move || {
            loop {
                if video.is_disconnected() && audio.is_disconnected() {
                    break;
                }

                match stream.recv()? {
                    Msg::Video(pack) => {
                        if let Err(err) = video.try_send(pack) {
                            tracing::warn!("Dropped a video sample: {err}");
                        }
                    }
                    Msg::Audio(pack) => {
                        if let Err(err) = audio.try_send(pack) {
                            tracing::warn!("Dropped an audio sample: {err}");
                        }
                    }
                    Msg::Text(_) => {}
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

    /// Pop the next [`video::Spec`] from the queue, if present.
    pub fn video(&self) -> Result<video::Pack, flume::TryRecvError> {
        self.video.try_recv()
    }

    /// Pop the next [`audio::Spec`] from the queue, if present.
    pub fn audio(&self) -> Result<audio::Pack, flume::TryRecvError> {
        self.audio.try_recv()
    }
}

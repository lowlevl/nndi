use std::{
    net::{SocketAddr, TcpStream},
    thread,
};

use binrw::BinRead;
use mdns_sd::ServiceInfo;

use crate::{
    frame::Frame,
    msg::{audio, metadata::Metadata, video, Msg},
    Result,
};

pub struct Recv {
    video: flume::Receiver<video::Pack>,
    audio: flume::Receiver<audio::Pack>,
}

impl Recv {
    pub fn new(service: &ServiceInfo, queue: usize) -> Result<Self> {
        let port = service.get_port();
        let stream = TcpStream::connect(
            &*service
                .get_addresses()
                .iter()
                .map(|addr| SocketAddr::new(*addr, port))
                .collect::<Vec<_>>(),
        )?;

        let (videotx, video) = flume::bounded(queue);
        let (audiotx, audio) = flume::bounded(queue);
        Self::task(stream, videotx, audiotx);

        Ok(Self { video, audio })
    }

    fn task(
        stream: TcpStream,
        video: flume::Sender<video::Pack>,
        audio: flume::Sender<audio::Pack>,
    ) {
        let task = move || {
            let mut stream = binrw::io::NoSeek::new(stream);

            loop {
                if video.is_disconnected() && audio.is_disconnected() {
                    break;
                }

                match Frame::read(&mut stream)?.unpack()? {
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
                    Msg::Text(pack) => {
                        let Ok(info) = Metadata::from_pack(&pack) else {
                            tracing::warn!(
                                "Unhandled information: {}",
                                String::from_utf8_lossy(&pack.data)
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
                tracing::error!("Fatal error in the `Recv::task` thread: {err}");
            }
        });
    }

    /// Pop the next [`video::Spec`] from the queue, if present.
    pub fn video(&self) -> Option<video::Pack> {
        self.video.try_recv().ok()
    }

    /// Pop the next [`audio::Spec`] from the queue, if present.
    pub fn audio(&self) -> Option<audio::Pack> {
        self.audio.try_recv().ok()
    }
}

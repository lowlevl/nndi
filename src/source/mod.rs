use std::sync::{Arc, Weak};

use ffmpeg::codec;
use futures::{StreamExt, TryFutureExt};
use mdns_sd::{ServiceDaemon, ServiceInfo, UnregisterStatus};
use tokio::{net::TcpListener, sync::RwLock};

use crate::{
    io::{frame::text, Stream},
    Error, Result,
};

mod config;
pub use config::Config;

mod peer;
pub use peer::Peer;

type Lock<T> = Arc<RwLock<T>>;
type WeakLock<T> = Weak<RwLock<T>>;

/// A _video_ and _audio_ source, that can send data to multiple sinks.
pub struct Source {
    name: String,
    mdns: ServiceDaemon,

    peers: Lock<Vec<WeakLock<Peer>>>,
}

impl Source {
    pub async fn new(config: Config<'_>) -> Result<Self> {
        let groups = config.groups.unwrap_or(&["public"]).join(",");
        let listener = TcpListener::bind("[::]:0").await?;

        let mdns = ServiceDaemon::new()?;
        let service = ServiceInfo::new(
            super::SERVICE_TYPE,
            &crate::name(config.name),
            &crate::hostname(),
            (),
            listener.local_addr()?.port(),
            [("groups", groups.as_str())].as_slice(),
        )?
        .enable_addr_auto();

        let name = service.get_fullname().into();
        mdns.register(service)?;

        tracing::debug!("Registered mDNS service `{}`", name);

        let peers = <Lock<Vec<WeakLock<Peer>>>>::default();
        tokio::spawn(
            Self::listen(listener, peers.clone())
                .inspect_err(|err| tracing::error!("Fatal error in `Source::listener`: {err}")),
        );

        Ok(Self { name, mdns, peers })
    }

    async fn listen(listener: tokio::net::TcpListener, peers: Lock<Vec<WeakLock<Peer>>>) -> Result {
        let mut streams: Vec<(Lock<Peer>, Stream)> = Vec::new();

        loop {
            tokio::select! {
                accepted = listener.accept() => {
                    let (stream, addr) = accepted?;
                    let mut stream = stream.into();

                    let peer = tokio::time::timeout(
                        crate::HANDSHAKE_TIMEOUT,
                        Peer::handshake(addr, &mut stream)
                    )
                    .await??;
                    let peer = Arc::from(RwLock::new(peer));

                    peers.write().await.push(Arc::downgrade(&peer));
                    streams.push((peer, stream));
                }
                Some(stream) = async {
                    let mut readable = streams
                        .iter_mut()
                        .map(|(peer, stream)| async { stream.readable().await.map(|_| (peer, stream)) })
                        .collect::<futures::stream::FuturesUnordered<_>>();

                    readable.next().await
                } => {
                    let res: Result = async {
                        let (peer, stream) = stream?;

                        if let Some(text::Metadata::Tally(tally)) = stream.metadata().await? {
                            peer.write().await.tally = tally;
                        }

                        Ok(())
                    }.await;

                    if let Err(err) = res {
                       tracing::error!("Peer handling failed: {err}");

                        //TODO: handle disconnect
                    }
                }
            }
        }
    }

    /// List the peers currently connected to the [`Source`], with their parameters.
    pub async fn peers(&self) -> Vec<Peer> {
        let pointers: Vec<_> = self
            .peers
            .read()
            .await
            .iter()
            .filter_map(Weak::upgrade)
            .collect();

        let peers = futures::future::join_all(
            pointers
                .iter()
                .map(|peer| async { peer.read().await.clone() }),
        )
        .await;

        *self.peers.write().await = pointers.iter().map(Arc::downgrade).collect();

        peers
    }

    /// Get current _tally_ information computed from all the connected peers of the [`Source`].
    pub async fn tally(&self) -> text::Tally {
        self.peers()
            .await
            .into_iter()
            .fold(Default::default(), |current, peer| current | peer.tally)
    }

    /// Broadcast a [`ffmpeg::frame::Video`] to all the connected peers.
    pub fn broadcast_video(
        &self,
        frame: &ffmpeg::frame::Video,
        timebase: ffmpeg::sys::AVRational,
    ) -> Result {
        let mut converted = ffmpeg::frame::Video::new(
            ffmpeg::format::Pixel::YUV422P,
            frame.width(),
            frame.height(),
        );
        frame
            .converter(converted.format())?
            .run(frame, &mut converted)?;

        let mut context = codec::Context::new();
        // SAFETY: The pointer is allocated on the line before,
        // and is guaranteed to be exclusive with `as_mut_ptr`.
        unsafe {
            (*context.as_mut_ptr()).time_base = timebase;
            (*context.as_mut_ptr()).pix_fmt = converted.format().into();
            (*context.as_mut_ptr()).width = converted.width() as i32;
            (*context.as_mut_ptr()).height = converted.height() as i32;
        }

        let mut encoder = context
            .encoder()
            .video()?
            .open_as(codec::encoder::find(codec::Id::SPEEDHQ))?;

        encoder.send_frame(&converted)?;
        encoder.send_eof()?;

        let mut packet = ffmpeg::Packet::empty();
        encoder.receive_packet(&mut packet)?;

        tracing::error!("PAK SIZE: {:?}", packet.data().map(<[u8]>::len));

        Ok(())
    }

    /// Broadcast a [`ffmpeg::frame::Audio`] to all the connected peers.
    pub fn broadcast_audio(&self, frame: &ffmpeg::frame::Audio) -> Result {
        todo!("Broadcast an audio frame")
    }
}

impl Drop for Source {
    fn drop(&mut self) {
        match self.mdns.unregister(&self.name).map(|recv| recv.recv()) {
            Err(err) => tracing::error!(
                "Error while unregistering service `{}` from mDNS: {err}",
                self.name
            ),
            Ok(Err(err)) => tracing::error!(
                "Error while unregistering service `{}` from mDNS: {err}",
                self.name
            ),
            Ok(Ok(err @ UnregisterStatus::NotFound)) => tracing::error!(
                "Error while unregistering service `{}` from mDNS: {err:?}",
                self.name
            ),

            _ => tracing::debug!("Unregistered mDNS service `{}`", self.name),
        }

        if let Err(err) = self.mdns.shutdown() {
            tracing::error!("Error while shutting down the mDNS advertisement thread: {err}");
        }
    }
}

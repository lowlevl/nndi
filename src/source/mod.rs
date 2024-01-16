use std::{net::TcpListener, thread};

use ffmpeg::codec;
use mdns_sd::{ServiceDaemon, ServiceInfo, UnregisterStatus};

use crate::{io::Stream, Result};

mod config;
pub use config::Config;

mod peer;
use peer::Peer;

/// A _video_ and _audio_ source, that can send data to multiple sinks.
pub struct Source {
    name: String,
    mdns: ServiceDaemon,
}

impl Source {
    pub fn new(config: Config<'_>) -> Result<Self> {
        let groups = config.groups.unwrap_or(&["public"]).join(",");
        let listener = TcpListener::bind("[::]:0")?;

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

        Self::listen(listener);

        Ok(Self { name, mdns })
    }

    fn listen(listener: TcpListener) {
        let task = move || loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    let mut stream = stream.into();
                    let peer = match Peer::handshake(&mut stream, std::time::Duration::from_secs(3))
                    {
                        Ok(peer) => peer,
                        Err(err) => {
                            tracing::warn!(
                                "Unable to perform handshake with peer at `{addr}`: {err}",
                            );

                            continue;
                        }
                    };

                    Self::peer(peer, stream)
                }
                Err(err) => tracing::error!("Error while accepting connection: {err}"),
            }
        };

        thread::spawn(task);
    }

    fn peer(mut peer: Peer, mut stream: Stream) {
        let mut task = move || -> Result<()> {
            loop {
                peer.tick(&mut stream)?;
            }
        };

        thread::spawn(move || {
            if let Err(err) = task() {
                tracing::error!("Fatal error in the `Source::peer` thread: {err}");
            }
        });
    }

    pub fn send_video(
        &self,
        frame: &ffmpeg::frame::Video,
        timebase: ffmpeg::sys::AVRational,
    ) -> Result<()> {
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

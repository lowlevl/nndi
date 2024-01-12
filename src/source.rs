use std::{net::TcpListener, str, thread};

use ffmpeg::codec;
use mdns_sd::{ServiceDaemon, ServiceInfo, UnregisterStatus};

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

/// A _video_ and _audio_ source, that can send data to multiple sinks.
pub struct Source {
    mdns: ServiceDaemon,
    name: String,
}

impl Source {
    pub fn new(name: &str, groups: Option<&[&str]>) -> Result<Self> {
        let groups = groups.unwrap_or(&["public"]).join(",");
        let listener = TcpListener::bind("[::]:0")?;

        let mdns = ServiceDaemon::new()?;
        let service = ServiceInfo::new(
            super::SERVICE_TYPE,
            &crate::name(name),
            &crate::hostname(),
            (),
            listener.local_addr()?.port(),
            [("groups", groups.as_str()), ("discovery", "5960")].as_slice(),
        )?
        .enable_addr_auto();

        let name = service.get_fullname().into();
        mdns.register(service)?;

        tracing::debug!("Registered mDNS service `{}`", name);

        Self::task(listener);

        Ok(Self { mdns, name })
    }

    fn identify(stream: &mut Stream) -> Result<()> {
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

    fn task(listener: TcpListener) {
        thread::spawn(move || {
            for peer in listener.incoming() {
                match peer {
                    Err(err) => {
                        tracing::error!("Error while accepting connection: {err}");
                    }
                    Ok(stream) => {
                        thread::spawn(move || {
                            if let Err(err) = Self::peer(stream.into()) {
                                tracing::error!("Fatal error in the `Send::task` thread: {err}");
                            }
                        });
                    }
                }
            }
        });
    }

    fn peer(mut stream: Stream) -> Result<()> {
        tracing::info!("New peer connected from `{}`", stream.peer_addr()?);

        Self::identify(&mut stream)?;

        loop {
            match stream.recv()? {
                Frame::Video(_) | Frame::Audio(_) => (),
                Frame::Text(block) => {
                    let Ok(info) = Metadata::from_block(&block) else {
                        tracing::warn!(
                            "Unhandled information: {}",
                            String::from_utf8_lossy(&block.data)
                        );

                        continue;
                    };

                    tracing::warn!("Received information: {info:?}");

                    if let Metadata::Tally(tally) = info {
                        stream.send(Metadata::TallyEcho(tally).to_block()?)?
                    }
                }
            }
        }
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

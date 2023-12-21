use std::{net::TcpListener, str};

use mdns_sd::{ServiceDaemon, ServiceInfo, UnregisterStatus};

use crate::{
    io::{
        frame::{text::Metadata, Frame},
        Stream,
    },
    Result,
};

pub struct Send {
    mdns: ServiceDaemon,
    name: String,
}

impl Send {
    pub fn new(name: &str, groups: Option<&[&str]>) -> Result<Self> {
        let groups = groups.unwrap_or(&["public"]).join(",");
        let listener = TcpListener::bind("[::]:0")?;

        let mdns = ServiceDaemon::new()?;
        let service = ServiceInfo::new(
            super::SERVICE_TYPE,
            &crate::name(name)?,
            &crate::hostname()?,
            (),
            listener.local_addr()?.port(),
            [("groups", groups.as_str()), ("discovery", "5960")].as_slice(),
        )?
        .enable_addr_auto();

        let name = service.get_fullname().into();

        mdns.register(service.clone())?;

        tracing::debug!("Registered mDNS service `{}`", name);

        std::thread::spawn(move || {
            for peer in listener.incoming() {
                match peer {
                    Err(err) => {
                        tracing::error!("Error while accepting connection: {err}");
                    }
                    Ok(stream) => {
                        if let Err(err) = Self::peer(Stream::from(stream)) {
                            tracing::error!("Error while handling peer: {err}");
                        }
                    }
                }
            }
        });

        Ok(Self { mdns, name })
    }

    fn peer(mut stream: Stream) -> Result<()> {
        tracing::info!("New peer connected from `{}`", stream.peer_addr()?);

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
                }
            }
        }
    }
}

impl Drop for Send {
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

use std::{
    net::{TcpListener, TcpStream},
    os::unix::ffi::OsStrExt,
    str,
};

use binrw::{BinRead, BinWrite};
use mdns_sd::{ServiceDaemon, ServiceInfo, UnregisterStatus};

use crate::{
    frame::Frame,
    msg::{metadata::Metadata, video, Msg, Wrap},
    Error, Result,
};

pub struct Send {
    name: String,
    mdns: ServiceDaemon,
}

impl Send {
    pub fn new(name: &str, groups: Option<&[&str]>) -> Result<Self> {
        let groups = groups.unwrap_or(&["public"]).join(",");

        let hostname = gethostname::gethostname();
        let hostname = str::from_utf8(hostname.as_bytes()).map_err(Error::Hostname)?;

        let listener = TcpListener::bind("[::]:0")?;

        let mdns = ServiceDaemon::new()?;
        let service = ServiceInfo::new(
            super::SERVICE_TYPE,
            &format!("{hostname} ({name})"),
            hostname,
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
                        if let Err(err) = Self::peer(stream) {
                            tracing::error!("Error while handling peer: {err}");
                        }
                    }
                }
            }
        });

        Ok(Self { name, mdns })
    }

    fn peer(stream: TcpStream) -> Result<()> {
        tracing::info!("New peer connected from `{}`", stream.peer_addr()?);

        let mut stream = binrw::io::NoSeek::new(&stream);

        let msg = Msg::Video(Wrap::new(
            video::Spec {
                fourcc: crate::msg::video::FourCCVideoType::RGBX,
                width: 1,
                height: 1,
                fps_num: 30,
                fps_den: 1,
            },
            binrw::NullString(vec![u8::MAX, 0, 0]),
        ));
        Frame::pack(&msg)?.write(&mut stream)?;

        loop {
            let frame = Frame::read(&mut stream)?;

            match frame.unpack()? {
                Msg::Video(_) | Msg::Audio(_) => (),
                Msg::Text(text) => {
                    let text = text.data.0;

                    let Ok(info) =
                        quick_xml::de::from_reader::<_, Metadata>(&mut std::io::Cursor::new(&text))
                    else {
                        tracing::warn!("Unhandled information: {}", String::from_utf8_lossy(&text));

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
    }
}

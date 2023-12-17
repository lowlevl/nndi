use std::collections::HashMap;

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};

use crate::Result;

pub struct Scan {
    mdns: ServiceDaemon,
    receiver: mdns_sd::Receiver<ServiceEvent>,
    sources: HashMap<String, ServiceInfo>,
}

impl Scan {
    pub fn new() -> Result<Self> {
        let mdns = ServiceDaemon::new()?;
        let receiver = mdns.browse(super::SERVICE_TYPE)?;

        Ok(Self {
            mdns,
            receiver,
            sources: Default::default(),
        })
    }

    pub fn sources(&mut self) -> impl Iterator<Item = &ServiceInfo> {
        for event in self.receiver.try_iter() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    self.sources.insert(info.get_fullname().to_string(), info);
                }
                ServiceEvent::ServiceRemoved(_, name) => {
                    self.sources.remove(&name);
                }
                _ => (),
            }
        }

        self.sources.values()
    }
}

impl Drop for Scan {
    fn drop(&mut self) {
        if let Err(err) = self.mdns.stop_browse(super::SERVICE_TYPE) {
            tracing::error!(
                "Error while stopping the mDNS discovery of {}: {err}",
                super::SERVICE_TYPE
            );
        }

        if let Err(err) = self.mdns.shutdown() {
            tracing::error!("Error while shutting down the mDNS discovery thread: {err}");
        }
    }
}

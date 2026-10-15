//! Scoped DNS-SD on Windows/Linux; macOS retains its system Bonjour adapter.
use crate::platform::NetworkEvidence;
use anyhow::Result;
use mdns_sd::{DaemonEvent, IfKind, ServiceDaemon, ServiceEvent, ServiceInfo};
use std::net::Ipv4Addr;
use tokio::sync::mpsc;
pub enum DiscoveryEvent {
    Resolved {
        peer: String,
        certificate: String,
        address: Ipv4Addr,
        port: u16,
    },
    Removed(String),
    Failure(String),
}
pub struct Discovery {
    pub receiver: mpsc::Receiver<DiscoveryEvent>,
    daemon: ServiceDaemon,
    task: tokio::task::JoinHandle<()>,
}
impl Discovery {
    pub fn start(
        evidence: &NetworkEvidence,
        peer: &str,
        certificate: &str,
        port: u16,
    ) -> Result<Self> {
        #[cfg(feature = "test-profile")]
        let service = std::env::var("LOCAL_CHAT_TEST_SERVICE")
            .unwrap_or_else(|_| "_localchat-test._tcp".into());
        #[cfg(not(feature = "test-profile"))]
        let service = "_localchat._tcp".to_owned();
        anyhow::ensure!(
            service.starts_with('_')
                && service.ends_with("._tcp")
                && service.len() <= 21
                && service
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b)),
            "Invalid discovery service"
        );
        let service = format!("{service}.local.");
        let daemon = ServiceDaemon::new()?;
        // Queue scoping before registering or browsing. IPv6 is explicitly excluded.
        let initialized = (|| -> Result<_> {
            daemon.disable_interface(IfKind::All)?;
            let index = netdev::get_interfaces()
                .into_iter()
                .find(|i| i.name == evidence.interface)
                .ok_or_else(|| anyhow::anyhow!("Network interface disappeared"))?
                .index;
            daemon.enable_interface(IfKind::IndexV4(index))?;
            let info = ServiceInfo::new(
                &service,
                &format!("lc-{}", &peer[..16]),
                &format!("lc-{}.local.", &peer[..16]),
                evidence.address,
                port,
                &[
                    ("protocol", "localchat"),
                    ("version", "1"),
                    ("peer", peer),
                    ("cert", certificate),
                ][..],
            )?;
            let monitor = daemon.monitor()?;
            daemon.register(info)?;
            let events = daemon.browse(&service)?;
            Ok((events, monitor))
        })();
        let (events, monitor) = match initialized {
            Ok(value) => value,
            Err(e) => {
                let _ = daemon.shutdown();
                return Err(e);
            }
        };
        let (sender, receiver) = mpsc::channel(256);
        let subnet = evidence.clone();
        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    event = events.recv_async() => {
                        let event = match event {
                            Ok(ServiceEvent::ServiceResolved(info)) => {
                                if info.get_property_val_str("protocol") != Some("localchat") || info.get_property_val_str("version") != Some("1") { continue; }
                                let (Some(peer), Some(certificate)) = (info.get_property_val_str("peer"), info.get_property_val_str("cert")) else { continue; };
                                let Some(address) = info.get_addresses_v4().into_iter().filter(|a| subnet.permits(*a)).min() else { continue; };
                                DiscoveryEvent::Resolved { peer: peer.to_owned(), certificate: certificate.to_owned(), address, port: info.port }
                            }
                            Ok(ServiceEvent::ServiceRemoved(_, name)) => DiscoveryEvent::Removed(name),
                            Ok(_) => continue,
                            Err(_) => { let _ = sender.send(DiscoveryEvent::Failure("Discovery stopped".into())).await; break; }
                        };
                        if sender.try_send(event).is_err() { break; }
                    }
                    event = monitor.recv_async() => {
                        match event {
                            Ok(DaemonEvent::Error(_)) | Err(_) => { let _ = sender.send(DiscoveryEvent::Failure("Local multicast discovery is unavailable".into())).await; break; }
                            _ => {}
                        }
                    }
                }
            }
        });
        Ok(Self {
            receiver,
            daemon,
            task,
        })
    }
}
impl Drop for Discovery {
    fn drop(&mut self) {
        self.task.abort();
        let _ = self.daemon.shutdown();
    }
}

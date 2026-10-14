use crate::platform::NetworkEvidence;
use anyhow::Result;
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
}
impl Discovery {
    pub fn start(_: &NetworkEvidence, _: &str, _: &str, _: u16) -> Result<Self> {
        anyhow::bail!("Native discovery adapter has not yet been verified on this platform")
    }
}

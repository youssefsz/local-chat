#[cfg(target_os = "macos")]
use anyhow::Context as _;
use anyhow::Result;
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{net::Ipv4Addr, path::Path};

pub struct Identity {
    pub signing: SigningKey,
    pub cert: Vec<u8>,
    pub tls_key: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
struct Secret {
    signing: String,
    cert: String,
    tls_key: String,
}
impl Identity {
    pub fn id(&self) -> String {
        hex::encode(self.signing.verifying_key().as_bytes())
    }
    pub fn fingerprint(&self) -> String {
        hex::encode(Sha256::digest(&self.cert))
    }
    pub fn load(data: &Path, test: bool) -> Result<Self> {
        let path = data.join("test-identity.json");
        let entry = if test {
            None
        } else {
            Some(keyring::Entry::new(
                "app.localchat.desktop",
                "installation-v1",
            )?)
        };
        let saved = if let Some(entry) = &entry {
            match entry.get_password() {
                Ok(s) => Some(s),
                Err(keyring::Error::NoEntry) => None,
                Err(e) => return Err(e.into()),
            }
        } else {
            std::fs::read_to_string(&path).ok()
        };
        let secret: Secret = if let Some(saved) = saved {
            serde_json::from_str(&saved)?
        } else {
            let signing = SigningKey::generate(&mut rand::rngs::OsRng);
            let key = rcgen::KeyPair::generate()?;
            let cert =
                rcgen::CertificateParams::new(vec!["localchat.local".into()])?.self_signed(&key)?;
            let secret = Secret {
                signing: hex::encode(signing.to_bytes()),
                cert: hex::encode(cert.der()),
                tls_key: hex::encode(key.serialize_der()),
            };
            let json = serde_json::to_string(&secret)?;
            if let Some(entry) = &entry {
                entry.set_password(&json)?
            } else {
                std::fs::write(&path, json)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
                }
            }
            secret
        };
        let bytes: [u8; 32] = hex::decode(secret.signing)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid installation key"))?;
        Ok(Self {
            signing: SigningKey::from_bytes(&bytes),
            cert: hex::decode(secret.cert)?,
            tls_key: hex::decode(secret.tls_key)?,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkEvidence {
    // Set only after the OS adapter verifies physical interface evidence.
    physical: bool,
    pub interface: String,
    pub interface_index: u32,
    pub address: Ipv4Addr,
    pub mask: Ipv4Addr,
    pub gateway: Ipv4Addr,
    pub gateway_mac: Option<String>,
}
impl NetworkEvidence {
    pub fn fingerprint(&self) -> Option<String> {
        let mac = self.gateway_mac.as_ref()?;
        if !self.physical || self.address.is_loopback() || mac.is_empty() {
            return None;
        }
        let subnet = u32::from(self.address) & u32::from(self.mask);
        Some(hex::encode(Sha256::digest(format!(
            "localchat-context-v1|{}|{}|{}|{}",
            mac.to_lowercase(),
            self.gateway,
            subnet,
            self.mask
        ))))
    }
    pub fn permits(&self, address: Ipv4Addr) -> bool {
        !address.is_loopback()
            && (u32::from(address) & u32::from(self.mask))
                == (u32::from(self.address) & u32::from(self.mask))
    }
}
#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn localchat_gateway_hardware(
        interface: *const std::ffi::c_char,
        gateway: *const std::ffi::c_char,
        output: *mut std::ffi::c_char,
        capacity: usize,
    ) -> std::ffi::c_int;
    fn localchat_network_generation() -> u64;
    fn localchat_bind_interface(
        descriptor: std::ffi::c_int,
        name: *const std::ffi::c_char,
    ) -> std::ffi::c_int;
}
#[cfg(target_os = "macos")]
fn gateway_hardware(interface: &str, gateway: Ipv4Addr) -> Option<String> {
    let interface = std::ffi::CString::new(interface).ok()?;
    let gateway = std::ffi::CString::new(gateway.to_string()).ok()?;
    let mut output = [0i8; 64];
    // C writes at most capacity bytes and NUL-terminates on success. Inputs are owned CStrings.
    let ok = unsafe {
        localchat_gateway_hardware(
            interface.as_ptr(),
            gateway.as_ptr(),
            output.as_mut_ptr(),
            output.len(),
        )
    };
    if ok == 0 {
        return None;
    }
    let value = unsafe { std::ffi::CStr::from_ptr(output.as_ptr()) }
        .to_str()
        .ok()?;
    let bytes = value
        .split(':')
        .map(|part| u8::from_str_radix(part, 16))
        .collect::<std::result::Result<Vec<_>, _>>()
        .ok()?;
    if bytes.len() != 6 || bytes.iter().all(|b| *b == 0) {
        return None;
    }
    Some(
        bytes
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(":"),
    )
}
#[cfg(target_os = "macos")]
pub fn network_generation() -> u64 {
    unsafe { localchat_network_generation() }
}
#[cfg(any(target_os = "windows", target_os = "linux"))]
mod portable;
#[cfg(any(target_os = "windows", target_os = "linux"))]
pub use portable::{bind_physical_socket, network_evidence, network_generation};
#[cfg(target_os = "macos")]
pub fn bind_physical_socket(
    socket: &tokio::net::TcpSocket,
    evidence: &NetworkEvidence,
) -> Result<()> {
    use std::os::fd::AsRawFd;
    let name = std::ffi::CString::new(evidence.interface.as_str())?;
    // The borrowed socket descriptor stays valid for this synchronous setsockopt call.
    if unsafe { localchat_bind_interface(socket.as_raw_fd(), name.as_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}
#[cfg(target_os = "macos")]
pub fn network_evidence() -> Result<NetworkEvidence> {
    let out = std::process::Command::new("/sbin/route")
        .args(["-n", "get", "default"])
        .output()?;
    let route = String::from_utf8(out.stdout)?;
    let field = |name: &str| {
        route
            .lines()
            .find_map(|l| l.trim().strip_prefix(name).map(str::trim))
            .map(str::to_owned)
    };
    let interface = field("interface:").context("No default network interface")?;
    anyhow::ensure!(
        interface.starts_with("en"),
        "Waiting for a physical local network. VPN routes are not shared."
    );
    let gateway: Ipv4Addr = field("gateway:").context("No IPv4 gateway")?.parse()?;
    let iface = if_addrs::get_if_addrs()?
        .into_iter()
        .find(|i| i.name == interface && matches!(&i.addr, if_addrs::IfAddr::V4(_)))
        .context("No physical IPv4 address")?;
    let if_addrs::IfAddr::V4(v4) = iface.addr else {
        unreachable!()
    };
    let mac = gateway_hardware(&interface, gateway);
    Ok(NetworkEvidence {
        physical: true,
        interface_index: iface.index.context("Missing physical interface index")?,
        interface,
        address: v4.ip,
        mask: v4.netmask,
        gateway,
        gateway_mac: mac,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> NetworkEvidence {
        NetworkEvidence {
            physical: true,
            interface_index: 1,
            interface: "en0".into(),
            address: "192.168.0.2".parse().unwrap(),
            mask: "255.255.255.0".parse().unwrap(),
            gateway: "192.168.0.1".parse().unwrap(),
            gateway_mac: Some("00:01:02:03:04:05".into()),
        }
    }
    #[test]
    fn conservative_evidence() {
        let a = sample();
        let mut b = a.clone();
        b.address = "192.168.0.9".parse().unwrap();
        assert_eq!(a.fingerprint(), b.fingerprint());
        b.gateway_mac = Some("00:01:02:03:04:06".into());
        assert_ne!(a.fingerprint(), b.fingerprint());
        b.gateway_mac = None;
        assert!(b.fingerprint().is_none());
        b = sample();
        b.interface = "utun0".into();
        b.physical = false;
        assert!(b.fingerprint().is_none());
        assert!(!a.permits("10.0.0.2".parse().unwrap()));
    }
}

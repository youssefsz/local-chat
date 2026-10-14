//! Discovery normalizes platform records. It never receives message contents or local context IDs.
use crate::platform::NetworkEvidence;
use anyhow::Result;
use std::{
    ffi::{CStr, CString, c_char, c_void},
    net::Ipv4Addr,
};
use tokio::sync::mpsc;
#[derive(Debug)]
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
    handle: *mut c_void,
    callback: *mut mpsc::Sender<DiscoveryEvent>,
}
// The opaque C handle owns one pthread. Its only Rust operation is joining/stopping that thread.
// Callback data stays allocated until the join completes. There is no concurrent Rust handle access.
unsafe impl Send for Discovery {}
unsafe extern "C" {
    fn localchat_bonjour_start(
        interface: *const c_char,
        name: *const c_char,
        service_type: *const c_char,
        port: u16,
        txt: *const u8,
        txt_len: u16,
        callback: extern "C" fn(
            *mut c_void,
            i32,
            *const c_char,
            *const c_char,
            *const c_char,
            *const c_char,
            u16,
        ),
        user: *mut c_void,
    ) -> *mut c_void;
    fn localchat_bonjour_stop(handle: *mut c_void);
}
fn string(pointer: *const c_char) -> Option<String> {
    if pointer.is_null() {
        return None;
    }
    Some(unsafe { CStr::from_ptr(pointer) }.to_str().ok()?.to_owned())
}
extern "C" fn callback(
    user: *mut c_void,
    kind: i32,
    name: *const c_char,
    peer: *const c_char,
    cert: *const c_char,
    ip: *const c_char,
    port: u16,
) {
    if user.is_null() {
        return;
    }
    // The C adapter passes NUL-terminated strings owned for the callback duration.
    let event = match kind {
        0 => {
            let (Some(peer), Some(certificate), Some(ip)) =
                (string(peer), string(cert), string(ip))
            else {
                return;
            };
            let Ok(address) = ip.parse() else { return };
            DiscoveryEvent::Resolved {
                peer,
                certificate,
                address,
                port,
            }
        }
        1 => {
            let Some(name) = string(name) else { return };
            DiscoveryEvent::Removed(name)
        }
        2 => {
            let Some(message) = string(name) else { return };
            DiscoveryEvent::Failure(message)
        }
        _ => return,
    };
    let sender = unsafe { &*user.cast::<mpsc::Sender<DiscoveryEvent>>() };
    let _ = sender.try_send(event);
}
impl Discovery {
    pub fn start(
        evidence: &NetworkEvidence,
        peer: &str,
        certificate: &str,
        port: u16,
    ) -> Result<Self> {
        let interface = CString::new(evidence.interface.as_str())?;
        let name = CString::new(format!("lc-{}", &peer[..16]))?;
        #[cfg(feature = "test-profile")]
        let service_type = std::env::var("LOCAL_CHAT_TEST_SERVICE")
            .unwrap_or_else(|_| "_localchat-test._tcp".into());
        #[cfg(not(feature = "test-profile"))]
        let service_type = "_localchat._tcp".to_owned();
        anyhow::ensure!(
            service_type.ends_with("._tcp")
                && service_type.len() <= 21
                && service_type.starts_with('_')
                && service_type
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b)),
            "Invalid test discovery service"
        );
        let service_type = CString::new(service_type)?;
        let mut txt = Vec::new();
        for (key, value) in [
            ("protocol", "localchat"),
            ("version", "1"),
            ("peer", peer),
            ("cert", certificate),
        ] {
            let entry = format!("{key}={value}");
            txt.push(u8::try_from(entry.len())?);
            txt.extend(entry.bytes());
        }
        let (sender, receiver) = mpsc::channel(256);
        let user = Box::into_raw(Box::new(sender));
        let handle = unsafe {
            localchat_bonjour_start(
                interface.as_ptr(),
                name.as_ptr(),
                service_type.as_ptr(),
                port,
                txt.as_ptr(),
                u16::try_from(txt.len())?,
                callback,
                user.cast(),
            )
        };
        if handle.is_null() {
            drop(unsafe { Box::from_raw(user) });
            anyhow::bail!("Could not start native Bonjour discovery")
        }
        Ok(Self {
            receiver,
            handle,
            callback: user,
        })
    }
}
impl Drop for Discovery {
    fn drop(&mut self) {
        unsafe {
            localchat_bonjour_stop(self.handle);
            drop(Box::from_raw(self.callback));
        }
    }
}

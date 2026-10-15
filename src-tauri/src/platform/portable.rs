//! Windows/Linux OS evidence. Virtual default routes never authorize history export.
use super::NetworkEvidence;
use anyhow::{Context as _, Result, ensure};
use netdev::interface::types::InterfaceType;

unsafe extern "C" {
    fn localchat_network_generation() -> u64;
    #[cfg(target_os = "windows")]
    fn localchat_physical_interface(index: u32) -> i32;
}
pub fn network_generation() -> u64 {
    // One process-lifetime OS observer; callbacks update only an atomic generation.
    unsafe { localchat_network_generation() }
}
pub fn network_evidence() -> Result<NetworkEvidence> {
    ensure!(
        network_generation() != 0,
        "Network change monitoring is unavailable"
    );
    let interface = netdev::get_default_interface().map_err(anyhow::Error::msg)?;
    ensure!(
        interface.default,
        "The default route could not be confirmed"
    );
    let kind = matches!(
        interface.if_type,
        InterfaceType::Ethernet
            | InterfaceType::Wireless80211
            | InterfaceType::FastEthernetT
            | InterfaceType::FastEthernetFx
            | InterfaceType::GigabitEthernet
    );
    #[cfg(target_os = "windows")]
    let physical = unsafe { localchat_physical_interface(interface.index) != 0 };
    #[cfg(target_os = "linux")]
    let physical = std::fs::canonicalize(
        std::path::Path::new("/sys/class/net")
            .join(&interface.name)
            .join("device"),
    )
    .is_ok_and(|path| !path.starts_with("/sys/devices/virtual"));
    ensure!(
        kind && physical && interface.is_up() && !interface.is_tun() && !interface.is_loopback(),
        "Waiting for a physical Wi-Fi or Ethernet default route"
    );
    let gateway = interface.gateway.as_ref().context("No IPv4 gateway")?;
    // Exactly one gateway avoids guessing on multihomed or policy-routed setups.
    ensure!(gateway.ipv4.len() == 1, "The IPv4 gateway is ambiguous");
    let gateway_ip = gateway.ipv4[0];
    let mut addresses = interface.ipv4.iter().filter(|network| {
        network.contains(&gateway_ip)
            && !network.addr().is_loopback()
            && !network.addr().is_link_local()
    });
    let address = addresses
        .next()
        .context("No local IPv4 address for this gateway")?;
    ensure!(
        addresses.next().is_none(),
        "The local IPv4 address is ambiguous"
    );
    let mac = gateway.mac_addr.to_string().to_lowercase();
    let gateway_mac = (mac != "00:00:00:00:00:00" && mac != "ff:ff:ff:ff:ff:ff").then_some(mac);
    Ok(NetworkEvidence {
        interface: interface.name,
        address: address.addr(),
        mask: address.netmask(),
        gateway: gateway_ip,
        gateway_mac,
        physical: true,
    })
}

#[cfg(target_os = "linux")]
pub fn bind_physical_socket(socket: &tokio::net::TcpSocket, interface: &str) -> Result<()> {
    // A new socket can be bound once without elevated capabilities on supported kernels.
    // Failure is fatal to this session; never fall back to an unscoped socket.
    socket2::SockRef::from(socket).bind_device(Some(interface.as_bytes()))?;
    Ok(())
}
#[cfg(target_os = "windows")]
pub fn bind_physical_socket(socket: &tokio::net::TcpSocket, interface: &str) -> Result<()> {
    use std::os::windows::io::AsRawSocket;
    use windows_sys::Win32::Networking::WinSock::{
        IP_UNICAST_IF, IPPROTO_IP, SOCKET_ERROR, WSAGetLastError, setsockopt,
    };
    let index = netdev::get_interfaces()
        .into_iter()
        .find(|i| i.name == interface)
        .context("Network interface disappeared")?
        .index
        .to_be();
    // Winsock requires IP_UNICAST_IF in network byte order. Bind(local IPv4) follows at the caller.
    let result = unsafe {
        setsockopt(
            socket.as_raw_socket() as _,
            IPPROTO_IP,
            IP_UNICAST_IF,
            (&index as *const u32).cast(),
            std::mem::size_of::<u32>() as i32,
        )
    };
    if result == SOCKET_ERROR {
        return Err(std::io::Error::from_raw_os_error(unsafe { WSAGetLastError() }).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn os_observer_is_available() {
        assert_ne!(super::network_generation(), 0);
    }
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn new_socket_can_be_scoped_without_root() {
        let socket = tokio::net::TcpSocket::new_v4().unwrap();
        super::bind_physical_socket(&socket, "lo").unwrap();
        assert!(
            super::bind_physical_socket(&tokio::net::TcpSocket::new_v4().unwrap(), "lc-no-such-if")
                .is_err()
        );
    }
}

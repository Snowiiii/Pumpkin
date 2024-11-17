use pumpkin_config::{ADVANCED_CONFIG, BASIC_CONFIG};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time;

const BROADCAST_ADDRESS: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(224, 0, 2, 60)), 4445);

pub async fn start_lan_broadcast(bound_addr: SocketAddr) {
    let port = ADVANCED_CONFIG.lan_broadcast.port.unwrap_or(0);

    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port))
        .await
        .expect("Unable to bind to address");

    socket.set_broadcast(true).unwrap();

    let mut interval = time::interval(Duration::from_millis(1500));

    let motd: String;
    let advanced_motd = &ADVANCED_CONFIG
        .lan_broadcast
        .motd
        .clone()
        .unwrap_or_default();

    if advanced_motd.is_empty() {
        motd = BASIC_CONFIG.motd.replace('\n', " ");
        log::warn!("Using the server MOTD as the LAN broadcast MOTD. Note that the LAN broadcast MOTD does not support multiple lines, so consider defining it accordingly.");
    } else {
        motd = advanced_motd.clone();
    };

    let advertisement = format!("[MOTD]{}[/MOTD][AD]{}[/AD]", &motd, bound_addr.port());

    log::info!(
        "LAN broadcast running on {}",
        socket
            .local_addr()
            .expect("Unable to find running address!")
    );

    loop {
        interval.tick().await;
        let _ = socket
            .send_to(advertisement.as_bytes(), BROADCAST_ADDRESS)
            .await;
    }
}

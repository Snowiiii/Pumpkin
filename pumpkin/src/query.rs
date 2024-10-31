// Query protocol

use std::io::Cursor;

use pumpkin_protocol::query::SBasePacket;
use tokio::net::UdpSocket;

pub async fn start_query_handler() {
    let socket = UdpSocket::bind("0.0.0.0:25565")
        .await
        .expect("Unable to bind to address");
    log::info!("Query socket created");

    loop {
        let mut buf = vec![0; 1024];
        log::info!("Waiting for requests");
        let (len, addr) = socket.recv_from(&mut buf).await.unwrap();

        tokio::spawn(async move {
            let cursor = Cursor::new(buf);
            let packet = SBasePacket::decode(cursor).await;

            println!("{:#?}", packet);
        });
    }
}

// Query protocol

use std::{io::Cursor, sync::Arc};

use pumpkin_protocol::query::{CBasePacket, CBasePayload, PacketType, SBasePacket};
use rand::Rng;
use tokio::net::UdpSocket;

pub async fn start_query_handler() {
    let socket = Arc::new(
        UdpSocket::bind("0.0.0.0:25565")
            .await
            .expect("Unable to bind to address"),
    );
    log::info!("Query socket created");

    loop {
        let socket = socket.clone();
        let mut buf = vec![0; 1024];
        log::info!("Waiting for requests");
        let (len, addr) = socket.recv_from(&mut buf).await.unwrap();

        tokio::spawn(async move {
            let cursor = Cursor::new(buf);
            let packet = SBasePacket::decode(cursor).await;

            match packet.packet_type {
                PacketType::Handshake => {
                    let response = CBasePacket {
                        packet_type: PacketType::Handshake,
                        session_id: packet.session_id,
                        payload: CBasePayload::Handshake {
                            challange_token: rand::thread_rng().gen_range(1..=i32::MAX),
                        },
                    };

                    let _len = socket
                        .send_to(response.encode().await.as_slice(), addr)
                        .await
                        .unwrap();
                }
                PacketType::Stat => todo!(),
            }
        });
    }
}

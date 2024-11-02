// Query protocol

use std::{
    collections::HashMap, ffi::CString, io::Cursor, net::SocketAddr, sync::Arc, time::Duration,
};

use pumpkin_config::{ADVANCED_CONFIG, BASIC_CONFIG};
use pumpkin_protocol::query::{CBasePacket, CBasePayload, SBasePacket, SBasePayload};
use rand::Rng;
use tokio::{net::UdpSocket, sync::RwLock, time::interval};

use crate::server::{Server, CURRENT_MC_VERSION};

pub async fn start_query_handler(server: Arc<Server>, bound_addr: SocketAddr) {
    let mut bound_addr = bound_addr;
    if let Some(port) = ADVANCED_CONFIG.query.port {
        bound_addr.set_port(port);
    }

    let socket = Arc::new(
        UdpSocket::bind(bound_addr)
            .await
            .expect("Unable to bind to address"),
    );

    let clients = QueryClients::new();
    log::info!(
        "Server query running on {}",
        socket
            .local_addr()
            .expect("Unable to find running address!")
    );

    loop {
        let socket = socket.clone();
        let clients = clients.clone();
        let server = server.clone();
        let mut buf = vec![0; 1024];
        let (_, addr) = socket.recv_from(&mut buf).await.unwrap();

        tokio::spawn(async move {
            let cursor = Cursor::new(buf);

            if let Ok(packet) = SBasePacket::decode(cursor).await {

                match packet.payload {
                    SBasePayload::Handshake => {
                        let challange_token = rand::thread_rng().gen_range(1..=i32::MAX);
                        let response = CBasePacket {
                            session_id: packet.session_id,
                            payload: CBasePayload::Handshake { challange_token },
                        };

                        clients
                            .add_new_client(packet.session_id, challange_token, addr)
                            .await;

                        socket
                            .send_to(response.encode().await.as_slice(), addr)
                            .await
                            .unwrap();
                    },
                    SBasePayload::BasicInfo(challange_token) => {
                        if clients
                            .check_client(packet.session_id, challange_token, addr)
                            .await
                        {}
                    },
                    SBasePayload::FullInfo(challange_token) => {
                        if clients
                            .check_client(packet.session_id, challange_token, addr)
                            .await
                        {
                            let response = CBasePacket {
                                session_id: packet.session_id,
                                payload: CBasePayload::FullInfo {
                                    hostname: CString::new(BASIC_CONFIG.motd.as_str())
                                        .unwrap(),
                                    version: CString::new(CURRENT_MC_VERSION).unwrap(),
                                    plugins: CString::new("Pumpkin on 1.21.3").unwrap(), // TODO: Fill this with plugins when plugins are working
                                    map: CString::new("world").unwrap(), // TODO: Get actual world name
                                    num_players: server.get_player_count().await,
                                    max_players: BASIC_CONFIG.max_players as usize,
                                    host_port: bound_addr.port(),
                                    host_ip: CString::new(bound_addr.ip().to_string())
                                        .unwrap(),
                                    players: vec![], // TODO: Fill with players
                                },
                            };

                            socket
                                .send_to(response.encode().await.as_slice(), addr)
                                .await
                                .unwrap();
                        }
                    },
                }
            }
        });
    }
}

struct QueryClients {
    // Query by session id to get address and challange token
    // Clear hashmap every 30 seconds as thats how long every challange token ever lasts
    // If challange token is expired, the client needs to handshake again
    // So there is no point in keeping all this data
    clients: RwLock<HashMap<i32, (i32, SocketAddr)>>,
}

impl QueryClients {
    fn new() -> Arc<Self> {
        let clients = Arc::new(Self {
            clients: RwLock::new(HashMap::new()),
        });

        let clients_clone = Arc::clone(&clients);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            loop {
                interval.tick().await;
                clients_clone.clear_clients().await;
            }
        });

        clients
    }

    async fn add_new_client(&self, session_id: i32, challange_token: i32, addr: SocketAddr) {
        self.clients
            .write()
            .await
            .insert(session_id, (challange_token, addr));
    }

    async fn check_client(&self, session_id: i32, challange_token: i32, addr: SocketAddr) -> bool {
        if let Some(info) = self.clients.read().await.get(&session_id) {
            info.0 == challange_token && info.1 == addr
        } else {
            false
        }
    }

    async fn clear_clients(&self) {
        self.clients.write().await.clear();
    }
}

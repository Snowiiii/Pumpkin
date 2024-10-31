use tokio::io::AsyncReadExt;

#[derive(Debug)]
#[repr(u8)]
pub enum PacketType {
    Handshake = 9,
    Stat = 0,
}

#[derive(Debug)]
pub struct SBasePacket {
    pub magic: u16,
    pub packet_type: PacketType,
    pub session_id: i32,
    pub payload: SBasePayload,
}

#[derive(Debug)]
pub enum SBasePayload {
    Handshake,
    BasicInfo(i32),
    FullInfo(i32),
}

impl SBasePacket {
    pub async fn decode(mut reader: impl AsyncReadExt + Unpin) -> Self {
        let magic = reader.read_u16().await.unwrap();
        match reader.read_u8().await.unwrap() {
            0 => todo!(),
            // Handshake
            9 => Self {
                magic,
                packet_type: PacketType::Handshake,
                session_id: reader.read_i32().await.unwrap(),
                payload: SBasePayload::Handshake,
            },
            _ => todo!(),
        }
    }
}

pub struct CBasePacket {
    pub packet_type: PacketType,
    pub session_id: i32,
    pub payload: CBasePayload,
}

pub enum CBasePayload {
    Handshake {
        // For simplicity use a number type
        // Should be encoded as string here
        // Will be converted in encoding
        challange_token: i32,
    },
    BasicInfo {
        // Use CString as protocol requires nul terminated strings
        motd: String,
        gametype: String,
        map: String,
        num_players: String,
        max_players: String,
        host_port: u16,
        host_ip: String,
    },
    FullInfo {
        hostname: String,
        // Game type and game id are hardcoded into protocol
        // They are not here as they cannot be changed
        version: String,
        plugins: String,
        map: String,
        num_players: u16,
        max_players: u16,
        host_port: u16,
        host_ip: String,
        players: Vec<String>,
    },
}

impl CBasePacket {
    fn encode(&self) -> Vec<u8> {
        // let buf = Vec::new();

        match &self.payload {
            CBasePayload::Handshake { challange_token } => todo!(),
            CBasePayload::BasicInfo {
                motd,
                gametype,
                map,
                num_players,
                max_players,
                host_port,
                host_ip,
            } => todo!(),
            CBasePayload::FullInfo {
                hostname,
                version,
                plugins,
                map,
                num_players,
                max_players,
                host_port,
                host_ip,
                players,
            } => todo!(),
        }
    }
}

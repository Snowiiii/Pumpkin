use std::ffi::CString;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

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
    // We don't care what error it is as any packet with errors will be ingnored
    pub async fn decode(mut reader: impl AsyncReadExt + Unpin) -> Result<Self, ()> {
        let magic = reader.read_u16().await.map_err(|_| ())?;
        let packet_type = reader.read_u8().await.map_err(|_| ())?;
        let session_id = reader.read_i32().await.map_err(|_| ())?;

        match packet_type {
            // Status
            0 => {
                let challange_token = reader.read_i32().await.map_err(|_| ())?;
                let mut buf = [0; 4];

                // If payload is padded to 8 bytes, the client is requesting full status response
                // In other terms, check if there are 4 extra bytes at the end
                // The extra bytes should be meaningless
                // Otherwise the client is requesting basic status response
                match reader.read(&mut buf).await {
                    Ok(0) => Ok(Self {
                        magic,
                        packet_type: PacketType::Stat,
                        session_id,
                        payload: SBasePayload::BasicInfo(challange_token),
                    }),
                    Ok(4) => Ok(Self {
                        magic,
                        packet_type: PacketType::Stat,
                        session_id,
                        payload: SBasePayload::FullInfo(challange_token),
                    }),
                    _ => {
                        // Just ingnore malformed packets or errors
                        Err(())
                    }
                }
            }

            // Handshake
            9 => Ok(Self {
                magic,
                packet_type: PacketType::Handshake,
                session_id,
                payload: SBasePayload::Handshake,
            }),

            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct CBasePacket {
    pub packet_type: PacketType,
    pub session_id: i32,
    pub payload: CBasePayload,
}

#[derive(Debug)]
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
    pub async fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match &self.payload {
            CBasePayload::Handshake { challange_token } => {
                // Packet Type
                buf.write_u8(9).await.unwrap();
                // Session ID
                buf.write_i32(self.session_id).await.unwrap();
                // Challange token
                // Use CString to add null terminator and ensure no null bytes in the middle of data
                // Unwrap here since there should be no errors with nulls in the middle of data
                let token = CString::new(challange_token.to_string()).unwrap();
                buf.extend_from_slice(token.as_bytes_with_nul());
            }
            CBasePayload::BasicInfo {
                motd,
                gametype,
                map,
                num_players,
                max_players,
                host_port,
                host_ip,
            } => {
                
            },
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

        buf
    }
}

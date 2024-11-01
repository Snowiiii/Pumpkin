use std::{ffi::CString, iter};

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
        motd: CString,
        // Game type is hardcoded
        map: CString,
        num_players: usize,
        max_players: usize,
        host_port: u16,
        host_ip: CString,
    },
    FullInfo {
        hostname: CString,
        // Game type and game id are hardcoded into protocol
        // They are not here as they cannot be changed
        version: CString,
        plugins: CString,
        map: CString,
        num_players: u16,
        max_players: u16,
        host_port: u16,
        host_ip: CString,
        players: Vec<CString>,
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
                map,
                num_players,
                max_players,
                host_port,
                host_ip,
            } => {
                // Packet Type
                buf.write_u8(0).await.unwrap();
                // Session ID
                buf.write_i32(self.session_id).await.unwrap();
                // MOTD
                buf.extend_from_slice(motd.as_bytes_with_nul());
                // Game Type
                let game_type = CString::new("SMP").unwrap();
                buf.extend_from_slice(game_type.as_bytes_with_nul());
                // Map
                buf.extend_from_slice(map.as_bytes_with_nul());
                // Num players
                let num_players = CString::new(num_players.to_string()).unwrap();
                buf.extend_from_slice(num_players.as_bytes_with_nul());
                // Max players
                let max_players = CString::new(max_players.to_string()).unwrap();
                buf.extend_from_slice(max_players.as_bytes_with_nul());
                // Port
                // No idea why the port needs to be in little endian
                buf.write_u16_le(*host_port).await.unwrap();
                // IP
                buf.extend_from_slice(host_ip.as_bytes_with_nul());
            }
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
            } => {
                // Packet type
                buf.write_u8(0).await.unwrap();
                // Session ID
                buf.write_i32(self.session_id).await.unwrap();
                // Padding (11 bytes, meaningless)
                buf.extend(iter::repeat(0).take(11));

                // Key value section
                buf.extend_from_slice(CString::new("hostname").unwrap().as_bytes_with_nul());
                buf.extend_from_slice(hostname.as_bytes_with_nul());

                // Game type and game id are hardcoded into protocol, these are not changeable
                // No idea why game type has a space in between, but it does apparently
                buf.extend_from_slice(CString::new("game type").unwrap().as_bytes_with_nul());
                buf.extend_from_slice(CString::new("SMP").unwrap().as_bytes_with_nul());

                // No idea why there is a underscore here, but it does apparently
                buf.extend_from_slice(CString::new("game_id").unwrap().as_bytes_with_nul());
                buf.extend_from_slice(CString::new("MINECRAFT").unwrap().as_bytes_with_nul());

                buf.extend_from_slice(CString::new("version").unwrap().as_bytes_with_nul());
                buf.extend_from_slice(version.as_bytes_with_nul());

                buf.extend_from_slice(CString::new("plugins").unwrap().as_bytes_with_nul());
                buf.extend_from_slice(plugins.as_bytes_with_nul());

                buf.extend_from_slice(CString::new("map").unwrap().as_bytes_with_nul());
                buf.extend_from_slice(map.as_bytes_with_nul());

                buf.extend_from_slice(CString::new("numplayers").unwrap().as_bytes_with_nul());
                let num_players = CString::new(num_players.to_string()).unwrap();
                buf.extend_from_slice(num_players.as_bytes_with_nul());

                buf.extend_from_slice(CString::new("maxplayers").unwrap().as_bytes_with_nul());
                let max_players = CString::new(max_players.to_string()).unwrap();
                buf.extend_from_slice(max_players.as_bytes_with_nul());

                buf.extend_from_slice(CString::new("hostport").unwrap().as_bytes_with_nul());
                let host_port = CString::new(host_port.to_string()).unwrap();
                buf.extend_from_slice(host_port.as_bytes_with_nul());

                buf.extend_from_slice(CString::new("hostip").unwrap().as_bytes_with_nul());
                buf.extend_from_slice(host_ip.as_bytes_with_nul());

                // Padding (10 bytes, meaningless), with one extra 0x00 for the extra required null terminator
                buf.extend(iter::repeat(0).take(11));

                // Players
                for player in players {
                    buf.extend_from_slice(player.as_bytes_with_nul());
                }
                // Required extra null terminator
                buf.write_u8(0).await.unwrap();
            }
        }

        buf
    }
}

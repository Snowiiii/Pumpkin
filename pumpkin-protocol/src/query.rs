use std::{ffi::CString, io::Cursor};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(FromPrimitive)]
#[repr(u8)]
pub enum PacketType {
    // There could be other types but they are not documented
    // Besides these types are enough to get server status
    Handshake = 9,
    Status = 0,
}

pub struct RawQueryPacket {
    pub packet_type: PacketType,
    reader: Cursor<Vec<u8>>,
}

impl RawQueryPacket {
    pub async fn decode(bytes: Vec<u8>) -> Result<Self, ()> {
        let mut reader = Cursor::new(bytes);

        match reader.read_u16().await.map_err(|_| ())? {
            // Magic should always equal 65277
            // Since it denotes the protocol being used
            // Should not attempt to decode packets with other magic values
            65277 => Ok(Self {
                packet_type: PacketType::from_u8(reader.read_u8().await.map_err(|_| ())?)
                    .ok_or(())?,
                reader,
            }),
            _ => Err(()),
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct SHandshake {
    pub session_id: i32,
}

impl SHandshake {
    pub async fn decode(packet: &mut RawQueryPacket) -> Result<Self, ()> {
        Ok(Self {
            session_id: packet.reader.read_i32().await.map_err(|_| ())?,
        })
    }
}

#[derive(PartialEq, Debug)]
pub struct SStatusRequest {
    pub session_id: i32,
    pub challenge_token: i32,
    // Full status request and basic status request are pretty much similar
    // So might as just use the same struct
    pub is_full_request: bool,
}

impl SStatusRequest {
    pub async fn decode(packet: &mut RawQueryPacket) -> Result<Self, ()> {
        Ok(Self {
            session_id: packet.reader.read_i32().await.map_err(|_| ())?,
            challenge_token: packet.reader.read_i32().await.map_err(|_| ())?,
            is_full_request: {
                let mut buf = [0; 4];

                // If payload is padded to 8 bytes, the client is requesting full status response
                // In other terms, check if there are 4 extra bytes at the end
                // The extra bytes should be meaningless
                // Otherwise the client is requesting basic status response
                match packet.reader.read(&mut buf).await {
                    Ok(0) => false,
                    Ok(4) => true,
                    _ => {
                        // Just ignore malformed packets or errors
                        return Err(());
                    }
                }
            },
        })
    }
}

pub struct CHandshake {
    pub session_id: i32,
    // For simplicity use a number type
    // Should be encoded as string here
    // Will be converted in encoding
    pub challenge_token: i32,
}

impl CHandshake {
    pub async fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Packet Type
        buf.write_u8(9).await.unwrap();
        // Session ID
        buf.write_i32(self.session_id).await.unwrap();
        // Challenge token
        // Use CString to add null terminator and ensure no null bytes in the middle of data
        // Unwrap here since there should be no errors with nulls in the middle of data
        let token = CString::new(self.challenge_token.to_string()).unwrap();
        buf.extend_from_slice(token.as_bytes_with_nul());

        buf
    }
}

pub struct CBasicStatus {
    pub session_id: i32,
    // Use CString as protocol requires nul terminated strings
    pub motd: CString,
    // Game type is hardcoded
    pub map: CString,
    pub num_players: usize,
    pub max_players: usize,
    pub host_port: u16,
    pub host_ip: CString,
}

impl CBasicStatus {
    pub async fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Packet Type
        buf.write_u8(0).await.unwrap();
        // Session ID
        buf.write_i32(self.session_id).await.unwrap();
        // MOTD
        buf.extend_from_slice(self.motd.as_bytes_with_nul());
        // Game Type
        let game_type = CString::new("SMP").unwrap();
        buf.extend_from_slice(game_type.as_bytes_with_nul());
        // Map
        buf.extend_from_slice(self.map.as_bytes_with_nul());
        // Num players
        let num_players = CString::new(self.num_players.to_string()).unwrap();
        buf.extend_from_slice(num_players.as_bytes_with_nul());
        // Max players
        let max_players = CString::new(self.max_players.to_string()).unwrap();
        buf.extend_from_slice(max_players.as_bytes_with_nul());
        // Port
        // No idea why the port needs to be in little endian
        buf.write_u16_le(self.host_port).await.unwrap();
        // IP
        buf.extend_from_slice(self.host_ip.as_bytes_with_nul());

        buf
    }
}

pub struct CFullStatus {
    pub session_id: i32,
    pub hostname: CString,
    // Game type and game id are hardcoded into protocol
    // They are not here as they cannot be changed
    pub version: CString,
    pub plugins: CString,
    pub map: CString,
    pub num_players: usize,
    pub max_players: usize,
    pub host_port: u16,
    pub host_ip: CString,
    pub players: Vec<CString>,
}

impl CFullStatus {
    pub async fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Packet type
        buf.write_u8(0).await.unwrap();
        // Session ID
        buf.write_i32(self.session_id).await.unwrap();

        // Padding (11 bytes, meaningless)
        // This is the padding used by vanilla
        // Although meaningless, in testing some query checkers depend on these bytes?
        const PADDING_START: [u8; 11] = [
            0x73, 0x70, 0x6C, 0x69, 0x74, 0x6E, 0x75, 0x6D, 0x00, 0x80, 0x00,
        ];
        buf.extend_from_slice(PADDING_START.as_slice());

        // Key-value pairs
        // Keys will not error when encoding as CString
        for (key, value) in [
            ("hostname", &self.hostname),
            ("gametype", &CString::new("SMP").unwrap()),
            ("game_id", &CString::new("MINECRAFT").unwrap()),
            ("version", &self.version),
            ("plugins", &self.plugins),
            ("map", &self.map),
            (
                "numplayers",
                &CString::new(self.num_players.to_string()).unwrap(),
            ),
            (
                "maxplayers",
                &CString::new(self.max_players.to_string()).unwrap(),
            ),
            (
                "hostport",
                &CString::new(self.host_port.to_string()).unwrap(),
            ),
            ("hostip", &self.host_ip),
        ] {
            buf.extend_from_slice(CString::new(key).unwrap().as_bytes_with_nul());
            buf.extend_from_slice(value.as_bytes_with_nul());
        }

        // Padding (10 bytes, meaningless), with one extra 0x00 for the extra required null terminator after the Key Value section
        const PADDING_END: [u8; 11] = [
            0x00, 0x01, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x5F, 0x00, 0x00,
        ];
        buf.extend_from_slice(PADDING_END.as_slice());

        // Players
        for player in &self.players {
            buf.extend_from_slice(player.as_bytes_with_nul());
        }
        // Required extra null terminator
        buf.write_u8(0).await.unwrap();

        buf
    }
}

// All test bytes/packets are from protocol documentation
#[tokio::test]
async fn test_handshake_request() {
    let bytes = vec![0xFE, 0xFD, 0x09, 0x00, 0x00, 0x00, 0x01];
    let mut raw_packet = RawQueryPacket::decode(bytes).await.unwrap();
    let packet = SHandshake::decode(&mut raw_packet).await.unwrap();

    // What the decoded packet should look like
    let actual_packet = SHandshake { session_id: 1 };

    assert_eq!(packet, actual_packet);
}

#[tokio::test]
async fn test_handshake_response() {
    let bytes = vec![
        0x09, 0x00, 0x00, 0x00, 0x01, 0x39, 0x35, 0x31, 0x33, 0x33, 0x30, 0x37, 0x00,
    ];

    let packet = CHandshake {
        session_id: 1,
        challenge_token: 9513307,
    };

    assert_eq!(bytes, packet.encode().await)
}

#[tokio::test]
async fn test_basic_stat_request() {
    let bytes = vec![
        0xFE, 0xFD, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x91, 0x29, 0x5B,
    ];
    let mut raw_packet = RawQueryPacket::decode(bytes).await.unwrap();
    let packet = SStatusRequest::decode(&mut raw_packet).await.unwrap();

    let actual_packet = SStatusRequest {
        session_id: 1,
        challenge_token: 9513307,
        is_full_request: false,
    };

    assert_eq!(packet, actual_packet);
}

#[tokio::test]
async fn test_basic_stat_response() {
    let bytes = vec![
        0x00, 0x00, 0x00, 0x00, 0x01, 0x41, 0x20, 0x4D, 0x69, 0x6E, 0x65, 0x63, 0x72, 0x61, 0x66,
        0x74, 0x20, 0x53, 0x65, 0x72, 0x76, 0x65, 0x72, 0x00, 0x53, 0x4D, 0x50, 0x00, 0x77, 0x6F,
        0x72, 0x6C, 0x64, 0x00, 0x32, 0x00, 0x32, 0x30, 0x00, 0xDD, 0x63, 0x31, 0x32, 0x37, 0x2E,
        0x30, 0x2E, 0x30, 0x2E, 0x31, 0x00,
    ];

    let packet = CBasicStatus {
        session_id: 1,
        motd: CString::new("A Minecraft Server").unwrap(),
        map: CString::new("world").unwrap(),
        num_players: 2,
        max_players: 20,
        host_port: 25565,
        host_ip: CString::new("127.0.0.1").unwrap(),
    };

    assert_eq!(bytes, packet.encode().await);
}

#[tokio::test]
async fn test_full_stat_request() {
    let bytes = vec![
        0xFE, 0xFD, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x91, 0x29, 0x5B, 0x00, 0x00, 0x00, 0x00,
    ];
    let mut raw_packet = RawQueryPacket::decode(bytes).await.unwrap();
    let packet = SStatusRequest::decode(&mut raw_packet).await.unwrap();

    let actual_packet = SStatusRequest {
        session_id: 1,
        challenge_token: 9513307,
        is_full_request: true,
    };

    assert_eq!(packet, actual_packet);
}
#[tokio::test]
async fn test_full_stat_response() {
    let bytes = vec![
        0x00, 0x00, 0x00, 0x00, 0x01, 0x73, 0x70, 0x6C, 0x69, 0x74, 0x6E, 0x75, 0x6D, 0x00, 0x80,
        0x00, 0x68, 0x6F, 0x73, 0x74, 0x6E, 0x61, 0x6D, 0x65, 0x00, 0x41, 0x20, 0x4D, 0x69, 0x6E,
        0x65, 0x63, 0x72, 0x61, 0x66, 0x74, 0x20, 0x53, 0x65, 0x72, 0x76, 0x65, 0x72, 0x00, 0x67,
        0x61, 0x6D, 0x65, 0x74, 0x79, 0x70, 0x65, 0x00, 0x53, 0x4D, 0x50, 0x00, 0x67, 0x61, 0x6D,
        0x65, 0x5F, 0x69, 0x64, 0x00, 0x4D, 0x49, 0x4E, 0x45, 0x43, 0x52, 0x41, 0x46, 0x54, 0x00,
        0x76, 0x65, 0x72, 0x73, 0x69, 0x6F, 0x6E, 0x00, 0x42, 0x65, 0x74, 0x61, 0x20, 0x31, 0x2E,
        0x39, 0x20, 0x50, 0x72, 0x65, 0x72, 0x65, 0x6C, 0x65, 0x61, 0x73, 0x65, 0x20, 0x34, 0x00,
        0x70, 0x6C, 0x75, 0x67, 0x69, 0x6E, 0x73, 0x00, 0x00, 0x6D, 0x61, 0x70, 0x00, 0x77, 0x6F,
        0x72, 0x6C, 0x64, 0x00, 0x6E, 0x75, 0x6D, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x73, 0x00,
        0x32, 0x00, 0x6D, 0x61, 0x78, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x73, 0x00, 0x32, 0x30,
        0x00, 0x68, 0x6F, 0x73, 0x74, 0x70, 0x6F, 0x72, 0x74, 0x00, 0x32, 0x35, 0x35, 0x36, 0x35,
        0x00, 0x68, 0x6F, 0x73, 0x74, 0x69, 0x70, 0x00, 0x31, 0x32, 0x37, 0x2E, 0x30, 0x2E, 0x30,
        0x2E, 0x31, 0x00, 0x00, 0x01, 0x70, 0x6C, 0x61, 0x79, 0x65, 0x72, 0x5F, 0x00, 0x00, 0x62,
        0x61, 0x72, 0x6E, 0x65, 0x79, 0x67, 0x61, 0x6C, 0x65, 0x00, 0x56, 0x69, 0x76, 0x61, 0x6C,
        0x61, 0x68, 0x65, 0x6C, 0x76, 0x69, 0x67, 0x00, 0x00,
    ];

    let packet = CFullStatus {
        session_id: 1,
        hostname: CString::new("A Minecraft Server").unwrap(),
        version: CString::new("Beta 1.9 Prerelease 4").unwrap(),
        plugins: CString::new("").unwrap(),
        map: CString::new("world").unwrap(),
        num_players: 2,
        max_players: 20,
        host_port: 25565,
        host_ip: CString::new("127.0.0.1").unwrap(),
        players: vec![
            CString::new("barneygale").unwrap(),
            CString::new("Vivalahelvig").unwrap(),
        ],
    };

    assert_eq!(bytes, packet.encode().await);
}

use crate::protocol::{bytebuf::buffer::ByteBuffer, VarInt};

pub struct SLoginStart {
    pub name: String, // 16
    pub uuid: uuid::Uuid,
}

impl SLoginStart {
    pub const PACKET_ID: VarInt = 0;

    pub fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            name: bytebuf.read_string_len(16).unwrap(),
            uuid: bytebuf.read_uuid().unwrap(),
        }
    }
}

pub struct SEncryptionResponse {
    pub shared_secret_length: VarInt,
    pub shared_secret: Vec<u8>,
    pub verify_token_length: VarInt,
    pub verify_token: Vec<u8>,
}

impl SEncryptionResponse {
    pub const PACKET_ID: VarInt = 1;

    pub fn read(bytebuf: &mut ByteBuffer) -> Self {
        let shared_secret_length = bytebuf.read_var_int().unwrap();
        let shared_secret = bytebuf.read_bytes(shared_secret_length as usize).unwrap();
        let verify_token_length = bytebuf.read_var_int().unwrap();
        let verify_token = bytebuf.read_bytes(shared_secret_length as usize).unwrap();
        Self {
            shared_secret_length,
            shared_secret,
            verify_token_length,
            verify_token,
        }
    }
}

pub struct SLoginPluginResponse<'a> {
    message_id: VarInt,
    successful: bool,
    data: Option<&'a [u8]>,
}

impl<'a> SLoginPluginResponse<'a> {
    pub const PACKET_ID: VarInt = 2;

    pub fn read(bytebuf: &mut ByteBuffer) -> Self {
        Self {
            message_id: bytebuf.read_var_int().unwrap(),
            successful: bytebuf.read_bool().unwrap(),
            data: None, // TODO
        }
    }
}

// Acknowledgement to the Login Success packet sent to the server.
pub struct SLoginAcknowledged {
    // empty
}

impl SLoginAcknowledged {
    pub const PACKET_ID: VarInt = 3;

    pub fn read(_bytebuf: &mut ByteBuffer) -> Self {
        Self {}
    }
}

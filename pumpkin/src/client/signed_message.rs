use pumpkin_protocol::bytebuf::ByteBuffer;

#[allow(dead_code)]
fn base_sign(
    sender: &crate::entity::player::Player,
    index: u32,
    salt: i64,
    time_stamp: i64,
    content: &str,
    // Looks weird right ?. Basically just the Previous messages and their signature
    previous_messages: &[&[u8]],
) -> ByteBuffer {
    let mut buf = ByteBuffer::empty();
    // link
    buf.put_uuid(&sender.gameprofile.id);
    buf.put_uuid(&sender.session_id.load().unwrap());
    buf.put_u32(index);
    // body
    buf.put_i64(salt);
    buf.put_i64(time_stamp);
    buf.put_u32(content.len() as u32);
    buf.put_string(content);
    buf.put_list(previous_messages, |v, p| v.put_slice(p));
    buf
}

#[allow(dead_code)]
fn sign_hashed() {}

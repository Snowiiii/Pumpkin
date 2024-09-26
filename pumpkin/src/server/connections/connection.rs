use std::cmp;
use std::{net::SocketAddr, pin::Pin};

use bytes::{BufMut, BytesMut};
use pumpkin_protocol::packet_codec::EncryptedCodec;
use serde::Serialize;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;

use tokio_util::codec::{Decoder, Encoder};

/// Necessary internals for encrypting and decrypting data in a session.
///
/// TODO: This current method makes use of too many buffers and could likely be reduced to only one buffer.
/// Although this is less than ideal, the implementation should be encapsulated enough that refactoring later won't cause too many waves.
struct EncryptedConnection {
    codec: EncryptedCodec,
    read_encrypted_buffer: bytes::BytesMut,
    read_decrypted_buffer: bytes::BytesMut,
}

impl EncryptedConnection {
    fn poll_read(
        &mut self,
        cx: &mut std::task::Context<'_>,
        reader: Pin<&mut impl AsyncRead>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut backing = [0; 1024];
        let mut encryption_view = tokio::io::ReadBuf::new(&mut backing);
        match reader.poll_read(cx, &mut encryption_view) {
            std::task::Poll::Ready(Ok(())) => {
                self.read_encrypted_buffer.put(encryption_view.filled());
                match self.codec.decode(&mut self.read_encrypted_buffer) {
                    Ok(Some(decoded_bytes)) => {
                        self.read_decrypted_buffer.put(decoded_bytes);
                        buf.put(
                            self.read_decrypted_buffer.split_to(cmp::min(
                                buf.remaining(),
                                self.read_decrypted_buffer.len(),
                            )),
                        );
                        std::task::Poll::Ready(Ok(()))
                    }
                    Ok(None) => std::task::Poll::Pending,
                    Err(e) => todo!(),
                }
            }
            std::task::Poll::Ready(Err(e)) => todo!(),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }

    fn poll_write(
        &mut self,
        cx: &mut std::task::Context<'_>,
        writer: Pin<&mut impl AsyncWrite>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let mut data = bytes::BytesMut::new();
        self.codec
            .encode(bytes::BytesMut::from(buf), &mut data)
            .expect("Encrypting data failed!");
        writer.poll_write(cx, &data)
    }
}

/// A maybe-encrypted wrapper for a connection.
///
/// Generic over any type of stream.
pub(crate) struct Connection<T> {
    stream: T,
    socket_addr: SocketAddr,
    encryption: Option<EncryptedConnection>,
}

impl<T> Connection<T> {
    pub fn new(tcp_stream: T, socket_addr: SocketAddr) -> Self {
        Self {
            stream: tcp_stream,
            socket_addr,
            encryption: None,
        }
    }

    /// Enable encryption by setting a pre-shared key.
    pub fn enable_encryption(&mut self, key: [u8; 16]) {
        self.encryption = Some(EncryptedConnection {
            codec: EncryptedCodec::new(key),
            read_encrypted_buffer: BytesMut::new(),
            read_decrypted_buffer: BytesMut::new(),
        });
    }

    /// TODO: with enabling encryption and disabling it, what do we do with the leftover buffer data?
    pub fn disable_encryption(&mut self) {
        self.encryption = None;
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.stream
    }

    pub fn inner(&self) -> &T {
        &self.stream
    }

    pub fn write(&mut self, message: impl Serialize) {}
}

impl<T> AsyncRead for Connection<T>
where
    T: AsyncRead + Unpin,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.get_mut();

        if let Some(encryption) = &mut this.encryption {
            return encryption.poll_read(cx, Pin::new(&mut this.stream), buf);
        }
        Pin::new(&mut this.stream).poll_read(cx, buf)
    }
}

impl<T> AsyncWrite for Connection<T>
where
    T: AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let this = self.get_mut();
        if let Some(encryption) = &mut this.encryption {
            return encryption.poll_write(cx, Pin::new(&mut this.stream), buf);
        }
        Pin::new(&mut this.stream).poll_write(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

    #[tokio::test]
    async fn plaintext_connection() {
        let (mut client, server) = tokio::io::duplex(1024);
        // The socket address is unused in this context
        let mut server = Connection::new(server, SocketAddr::from_str("192.168.0.1:0").unwrap());
        client.write(b"Hello, world!").await.unwrap();
        let mut bytes = bytes::BytesMut::new();
        server.read_buf(&mut bytes).await.unwrap();
        assert_eq!(bytes[..], b"Hello, world!"[..]);
    }

    #[tokio::test]
    async fn encrypted_connection() {
        let (client, server) = tokio::io::duplex(1024);
        // The socket address is unused in this context
        let mut server: Connection<tokio::io::DuplexStream> =
            Connection::new(server, SocketAddr::from_str("192.168.0.1:0").unwrap());
        let mut client = Connection::new(client, SocketAddr::from_str("192.168.0.1:0").unwrap());
        let key = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        server.enable_encryption(key.clone());
        client.enable_encryption(key.clone());

        let original_message = b"Hello, world!";
        client.write(original_message).await.unwrap();

        let mut bytes = bytes::BytesMut::new();
        server.read_buf(&mut bytes).await.unwrap();
        assert_eq!(&bytes[..], &original_message[..]);
    }
}

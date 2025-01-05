use std::{future::Future, net::SocketAddr, sync::Arc};

use log::trace;
use tokio::{io, net::UdpSocket, sync::RwLock};

use rand::prelude::*;

use crate::{message_status::MessageStatus, receiver::Receiver, MESSAGE_SIZE};

// Session Id (8 bytes)
// Sequence Number (2 bytes)
// Total Parts (2 bytes)
// Protocol Status (1 byte) 
// Compress (1 byte)

pub struct Sender {
    server_addr: String,
    _secure: bool,
    compress: bool,
    message_status: Arc<RwLock<MessageStatus>>,
    _public_key: Option<String>,
    _private_key: Option<String>
}

impl Sender {
    pub async fn new(server_addr: String) -> io::Result<Self> {  
        let secure: bool = false;
        let compress: bool = false;

        Ok(Self {
            server_addr,
            _secure: secure,
            compress: compress,
            message_status: Arc::new(RwLock::new(MessageStatus::NotSecured)),
            _public_key: None,
            _private_key: None
        })
    }

    pub async fn send_message(&self, message: &[u8]) -> io::Result<()> {
        let socket = UdpSocket::bind("127.0.0.1:0").await?;

        let total_chunks = (message.len() + MESSAGE_SIZE as usize - 1) / MESSAGE_SIZE as usize;

        let session_id = rand::thread_rng().gen_range(0..u64::MAX);
        let session_id_bytes = session_id.to_be_bytes();
        let total_parts = (total_chunks as u16).to_be_bytes();
        let compressed_bytes = [self.compress as u8];
        let message_status = [self.message_status.read().await.to_u8()];

        for (i, chunk) in message.chunks(MESSAGE_SIZE as usize).enumerate() {
            let sequence_number = (i as u16 + 1).to_be_bytes();
    
            let mut packet = Vec::new();

            packet.extend_from_slice(&sequence_number);
            packet.extend_from_slice(&total_parts);
            packet.extend_from_slice(&session_id_bytes);

            packet.extend_from_slice(&message_status);
            packet.extend_from_slice(&compressed_bytes);

            packet.extend_from_slice(chunk);
    
            socket.send_to(&packet, self.server_addr.as_str()).await?;

            trace!(
                "Sent chunk {}/{} to {}",
                i + 1,
                total_chunks,
                self.server_addr.as_str()
            );
        }

        let mut buf = vec![0u8; 24];
        let (len, _) = socket.recv_from(&mut buf).await?;
        let response = String::from_utf8_lossy(&buf[..len]);

        if response == format!("ACK {}", session_id) {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Invalid acknowledgment"))
        }
    }   

    pub async fn send_message_process_reply<F, Fut>(&self, message: &[u8], function: F) -> io::Result<()>    
    where
        F: Fn(Vec<u8>) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send {
        let socket = UdpSocket::bind("127.0.0.1:0").await?;

        let total_chunks = (message.len() + MESSAGE_SIZE as usize - 1) / MESSAGE_SIZE as usize;

        let session_id = rand::thread_rng().gen_range(0..u64::MAX);
        let session_id_bytes = session_id.to_be_bytes();
        let total_parts = (total_chunks as u16).to_be_bytes();
        let compressed_bytes = [self.compress as u8];
        let message_status = [self.message_status.read().await.to_u8()];

        for (i, chunk) in message.chunks(MESSAGE_SIZE as usize).enumerate() {
            let sequence_number = (i as u16 + 1).to_be_bytes();
    
            let mut packet = Vec::new();

            packet.extend_from_slice(&sequence_number);
            packet.extend_from_slice(&total_parts);
            packet.extend_from_slice(&session_id_bytes);

            packet.extend_from_slice(&message_status);
            packet.extend_from_slice(&compressed_bytes);

            packet.extend_from_slice(chunk);
    
            socket.send_to(&packet, self.server_addr.as_str()).await?;

            trace!(
                "Sent chunk {}/{} to {}",
                i + 1,
                total_chunks,
                self.server_addr.as_str()
            );
        }

        let mut buf = vec![0u8; 24];
        let (len, _) = socket.recv_from(&mut buf).await?;
        let response = String::from_utf8_lossy(&buf[..len]);

        if response == format!("ACK {}", session_id) {
            let socket = Arc::new(socket);
            let socket = socket.clone();
            Receiver::process_reply_external(function, socket).await;
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Invalid acknowledgment"))
        }
    }

    pub async fn send_message_get_reply(&self, message: &[u8]) -> io::Result<Vec<u8>> {
        let socket = UdpSocket::bind("127.0.0.1:0").await?;

        let total_chunks = (message.len() + MESSAGE_SIZE as usize - 1) / MESSAGE_SIZE as usize;

        let session_id = rand::thread_rng().gen_range(0..u64::MAX);
        let session_id_bytes = session_id.to_be_bytes();
        let total_parts = (total_chunks as u16).to_be_bytes();
        let compressed_bytes = [self.compress as u8];
        let message_status = [self.message_status.read().await.to_u8()];

        for (i, chunk) in message.chunks(MESSAGE_SIZE as usize).enumerate() {
            let sequence_number = (i as u16 + 1).to_be_bytes();
    
            let mut packet = Vec::new();

            packet.extend_from_slice(&sequence_number);
            packet.extend_from_slice(&total_parts);
            packet.extend_from_slice(&session_id_bytes);

            packet.extend_from_slice(&message_status);
            packet.extend_from_slice(&compressed_bytes);

            packet.extend_from_slice(chunk);
    
            socket.send_to(&packet, self.server_addr.as_str()).await?;

            trace!(
                "Sent chunk {}/{} to {}",
                i + 1,
                total_chunks,
                self.server_addr.as_str()
            );
        }

        let mut buf = vec![0u8; 24];
        let (len, _) = socket.recv_from(&mut buf).await?;
        let response = String::from_utf8_lossy(&buf[..len]);

        if response == format!("ACK {}", session_id) {
            let socket = Arc::new(socket);
            let socket = socket.clone();
            let mut buf = [0u8; 1500];
            let message = Receiver::get_message_external(socket.clone(), &mut buf).await;
            Ok(message)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Invalid acknowledgment"))
        }
    }

    /// Internal use only
    pub(crate) async fn send_message_external(
        message: &[u8], 
        socket: Arc<UdpSocket>, 
        addr: SocketAddr,
        compress: bool,
        _secure: bool,
        _public_key: Option<&String>) -> io::Result<()> {
        let total_chunks = (message.len() + MESSAGE_SIZE as usize - 1) / MESSAGE_SIZE as usize;

        let session_id = rand::thread_rng().gen_range(0..u64::MAX);
        let session_id_bytes = session_id.to_be_bytes();
        let total_parts = (total_chunks as u16).to_be_bytes();
        let compressed_bytes = [compress as u8];
        let message_status = [MessageStatus::NotSecured.to_u8()];

        for (i, chunk) in message.chunks(MESSAGE_SIZE as usize).enumerate() {
            let sequence_number = (i as u16 + 1).to_be_bytes();
    
            // Prepare the header: [sequence_number (2 bytes), total_parts (2 bytes), session_id (8 bytes)]
            let mut packet = Vec::new();
            packet.extend_from_slice(&sequence_number);
            packet.extend_from_slice(&total_parts);
            packet.extend_from_slice(&session_id_bytes);

            packet.extend_from_slice(&message_status);
            packet.extend_from_slice(&compressed_bytes);

            packet.extend_from_slice(chunk);
    
            socket.send_to(&packet, addr).await?;
        }

        Ok(())
    }
}

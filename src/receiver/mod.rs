use std::{future::Future, sync::Arc};

use ahash::RandomState;
use dashmap::DashMap;
use log::trace;
use tokio::{io, net::UdpSocket, sync::RwLock};

use crate::{message_status::MessageStatus, sender::Sender, HEADER_SIZE};

#[derive(Clone)]
pub struct Receiver {
    receiver: Arc<UdpSocket>,
    message_buffers: Arc<DashMap<u64, Vec<(usize, Vec<u8>)>, RandomState>>,
    _secure: bool,
    _compress: bool,
    _message_status: Arc<RwLock<MessageStatus>>,
    _public_key: Option<String>,
    _private_key: Option<String>
}

impl Receiver {
    pub async fn new(local_addr: &str) -> io::Result<Self> {
        let socket = UdpSocket::bind(local_addr).await?;

        let secure: bool = false;
        let compress: bool = false;

        Ok(Self {
            receiver: Arc::new(socket),
            message_buffers: Arc::new(DashMap::with_hasher(RandomState::new())),
            _secure: secure,
            _compress: compress,
            _message_status: Arc::new(RwLock::new(MessageStatus::NotSecured)),
            _public_key: None,
            _private_key: None
        })
    }

    pub async fn start_processing_function<F, Fut>(&self, function: F)
    where
        F: Fn(Vec<u8>) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send {
        let receiver = self.receiver.clone();
        let message_buffers = self.message_buffers.clone();
        let mut buf = vec![0u8; 1500];
        
        loop {
            if let Ok((len, addr)) = receiver.recv_from(&mut buf).await {
                let received_data = &buf[..len];

                // Parse sequence number and total parts
                let header = &received_data[..HEADER_SIZE];
                let sequence_number = u16::from_be_bytes([header[0], header[1]]) as usize;
                let total_parts = u16::from_be_bytes([header[2], header[3]]) as usize;
                let session_id = u64::from_be_bytes(
                    [
                        header[4], 
                        header[5], 
                        header[6],
                        header[7], 
                        header[8], 
                        header[9],
                        header[10],
                        header[11]
                    ]);

                let _message_status = header[12];
                let _compressed = header[13];

                let payload = received_data[HEADER_SIZE..].to_vec();

                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    sequence_number, total_parts, session_id, addr
                );

                // Add the chunk to the message buffer
                let mut buffer = message_buffers
                    .entry(session_id.clone())
                    .or_insert_with(Vec::new);

                buffer.push((sequence_number, payload));

                // If all chunks are received, reassemble the message
                if sequence_number == total_parts {
                    buffer.sort_by_key(|k| k.0);
                    let message: Vec<u8> = buffer.iter().flat_map(|(_, data)| data.clone()).collect();
                    
                    drop(buffer);

                    trace!(
                        "Reassembled message from session {}: {}",
                        session_id,
                        String::from_utf8_lossy(&message)
                    );

                    function(message).await;

                    message_buffers.remove(&session_id);

                    // Acknowledge the received chunk
                    let ack = format!("ACK {}", session_id);
                    _ = receiver.send_to(ack.as_bytes(), addr).await;
                }
            }
        }
    }

    pub async fn get_message(&self, buf: &mut [u8; 1500]) -> Vec<u8> {
        let receiver = self.receiver.clone();
        let message_buffers = self.message_buffers.clone();

        let message = loop {
            if let Ok((len, addr)) = receiver.recv_from(buf).await {
                let received_data = &buf[..len];
    
                // Parse sequence number and total parts
                let header = &received_data[..HEADER_SIZE];
                let sequence_number = u16::from_be_bytes([header[0], header[1]]) as usize;
                let total_parts = u16::from_be_bytes([header[2], header[3]]) as usize;
                let session_id = u64::from_be_bytes(
                    [
                        header[4], 
                        header[5], 
                        header[6],
                        header[7], 
                        header[8], 
                        header[9],
                        header[10],
                        header[11]
                    ]);
    
                let _message_status = header[12];
                let _compressed = header[13];
    
                let payload = received_data[HEADER_SIZE..].to_vec();
    
                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    sequence_number, total_parts, session_id, addr
                );
    
                // Add the chunk to the message buffer
                let mut buffer = message_buffers
                    .entry(session_id.clone())
                    .or_insert_with(Vec::new);
    
                buffer.push((sequence_number, payload));
    
                // If all chunks are received, reassemble the message
                if sequence_number == total_parts {
                    buffer.sort_by_key(|k| k.0);
                    let message: Vec<u8> = buffer.iter().flat_map(|(_, data)| data.clone()).collect();
                    
                    drop(buffer);
    
                    trace!(
                        "Reassembled message from session {}: {}",
                        session_id,
                        String::from_utf8_lossy(&message)
                    );
                    
                    message_buffers.remove(&session_id);
    
                    // Acknowledge the received chunk
                    let ack = format!("ACK {}", session_id);
                    _ = receiver.send_to(ack.as_bytes(), addr).await;
    
                    break message
                }
            }
        };

        message
    }

    pub async fn start_processing_function_result<F, Fut>(&self, function: F)
    where
        F: Fn(Vec<u8>) -> Fut + Send + Sync,
        Fut: Future<Output = Vec<u8>> + Send {
        let receiver = self.receiver.clone();
        let message_buffers = self.message_buffers.clone();
        let mut buf = vec![0u8; 1500];
        
        loop {
            if let Ok((len, addr)) = receiver.recv_from(&mut buf).await {
                let received_data = &buf[..len];

                // Parse sequence number and total parts
                let header = &received_data[..HEADER_SIZE];
                let sequence_number = u16::from_be_bytes([header[0], header[1]]) as usize;
                let total_parts = u16::from_be_bytes([header[2], header[3]]) as usize;
                let session_id = u64::from_be_bytes(
                    [
                        header[4], 
                        header[5], 
                        header[6],
                        header[7], 
                        header[8], 
                        header[9],
                        header[10],
                        header[11]
                    ]);

                let _message_status = header[12];
                let _compressed = header[13];

                let payload = received_data[HEADER_SIZE..].to_vec();

                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    sequence_number, total_parts, session_id, addr
                );

                // Add the chunk to the message buffer
                let mut buffer = message_buffers
                    .entry(session_id.clone())
                    .or_insert_with(Vec::new);

                buffer.push((sequence_number, payload));

                // If all chunks are received, reassemble the message
                if sequence_number == total_parts {
                    buffer.sort_by_key(|k| k.0);
                    let message: Vec<u8> = buffer.iter().flat_map(|(_, data)| data.clone()).collect();
                    
                    drop(buffer);

                    trace!(
                        "Reassembled message from session {}: {}",
                        session_id,
                        String::from_utf8_lossy(&message)
                    );

                    let result: Vec<u8> = function(message).await;

                    message_buffers.remove(&session_id);

                    // Acknowledge the received chunk
                    let ack = format!("ACK {}", session_id);
                    _ = receiver.send_to(ack.as_bytes(), addr).await;

                    _ = Sender::send_message_external(
                        &result, 
                        receiver.clone(), 
                        addr, 
                        self._secure, 
                        self._compress,
                        None).await;
                }
            }
        }
    }

    /// Internal use only
    pub(crate) async fn process_reply_external<F, Fut>(
        function: F, 
        receiver: Arc<UdpSocket>)
    where
        F: Fn(Vec<u8>) -> Fut + Send + Sync,
        Fut: Future<Output = ()> + Send {

        let message_buffers: DashMap<u64, Vec<(usize, Vec<u8>)>, RandomState> = DashMap::default();
        let mut buf = vec![0u8; 1500];
        
        loop {
            if let Ok((len, addr)) = receiver.recv_from(&mut buf).await {
                let received_data = &buf[..len];

                // Parse sequence number and total parts
                let header = &received_data[..HEADER_SIZE];
                let sequence_number = u16::from_be_bytes([header[0], header[1]]) as usize;
                let total_parts = u16::from_be_bytes([header[2], header[3]]) as usize;
                let session_id = u64::from_be_bytes(
                    [
                        header[4], 
                        header[5], 
                        header[6],
                        header[7], 
                        header[8], 
                        header[9],
                        header[10],
                        header[11]
                    ]);

                let _message_status = header[12];
                let _compressed = header[13];

                let payload = received_data[HEADER_SIZE..].to_vec();

                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    sequence_number, total_parts, session_id, addr
                );

                // Add the chunk to the message buffer
                let mut buffer = message_buffers
                    .entry(session_id.clone())
                    .or_insert_with(Vec::new);

                buffer.push((sequence_number, payload));

                // If all chunks are received, reassemble the message
                if sequence_number == total_parts {
                    buffer.sort_by_key(|k| k.0);
                    let message: Vec<u8> = buffer.iter().flat_map(|(_, data)| data.clone()).collect();
                    
                    drop(buffer);

                    trace!(
                        "Reassembled message from session {}: {}",
                        session_id,
                        String::from_utf8_lossy(&message)
                    );

                    function(message).await;

                    message_buffers.remove(&session_id);

                    // Acknowledge the received chunk
                    let ack = format!("ACK {}", session_id);
                    _ = receiver.send_to(ack.as_bytes(), addr).await;
                }
            }
        }
    }

    /// Internal use only
    pub(crate) async fn get_message_external(
        receiver: Arc<UdpSocket>, 
        buf: &mut [u8; 1500]) -> Vec<u8>{
        let message_buffers: DashMap<u64, Vec<(usize, Vec<u8>)>, RandomState> = DashMap::default();

        let message = loop {
            if let Ok((len, addr)) = receiver.recv_from(buf).await {
                let received_data = &buf[..len];
    
                // Parse sequence number and total parts
                let header = &received_data[..HEADER_SIZE];
                let sequence_number = u16::from_be_bytes([header[0], header[1]]) as usize;
                let total_parts = u16::from_be_bytes([header[2], header[3]]) as usize;
                let session_id = u64::from_be_bytes(
                    [
                        header[4], 
                        header[5], 
                        header[6],
                        header[7], 
                        header[8], 
                        header[9],
                        header[10],
                        header[11]
                    ]);
    
                let _message_status = header[12];
                let _compressed = header[13];
    
                let payload = received_data[HEADER_SIZE..].to_vec();
    
                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    sequence_number, total_parts, session_id, addr
                );
    
                // Add the chunk to the message buffer
                let mut buffer = message_buffers
                    .entry(session_id.clone())
                    .or_insert_with(Vec::new);
    
                buffer.push((sequence_number, payload));
    
                // If all chunks are received, reassemble the message
                if sequence_number == total_parts {
                    buffer.sort_by_key(|k| k.0);
                    let message: Vec<u8> = buffer.iter().flat_map(|(_, data)| data.clone()).collect();
                    
                    drop(buffer);
    
                    trace!(
                        "Reassembled message from session {}: {}",
                        session_id,
                        String::from_utf8_lossy(&message)
                    );
                    
                    message_buffers.remove(&session_id);
    
                    // Acknowledge the received chunk
                    let ack = format!("ACK {}", session_id);
                    _ = receiver.send_to(ack.as_bytes(), addr).await;
    
                    break message
                }
            }
        };

        message
    }
}
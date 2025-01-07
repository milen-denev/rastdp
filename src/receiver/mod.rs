use std::{future::Future, sync::Arc};

use ahash::RandomState;
use dashmap::DashMap;
use helpers::IncomingPacketInfo;
use log::trace;
use tokio::{io, net::UdpSocket, sync::RwLock};

use crate::{message_status::MessageStatus, sender::Sender, HEADER_SIZE};

pub(crate) mod helpers;

const ACK_BYTES: &[u8; 4]  = b"ACK ";

#[derive(Clone)]
pub struct Receiver {
    receiver: Arc<UdpSocket>,
    message_buffers: Arc<DashMap<u64, Vec<(u16, Vec<u8>)>, RandomState>>,
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

                if 
                    received_data[0] == ACK_BYTES[0] && 
                    received_data[1] == ACK_BYTES[1] &&
                    received_data[2] == ACK_BYTES[2] &&
                    received_data[3] == ACK_BYTES[3] {
                    return;
                }

                let incoming_packet_info = IncomingPacketInfo::new(received_data);
                let payload = received_data[HEADER_SIZE..].to_vec();

                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    incoming_packet_info.sequence_number, incoming_packet_info.total_parts, incoming_packet_info.session_id, addr
                );

                // Add the chunk to the message buffer
                let mut buffer = message_buffers
                    .entry(incoming_packet_info.session_id.clone())
                    .or_insert_with(Vec::new);

                buffer.push((incoming_packet_info.sequence_number, payload));

                // If all chunks are received, reassemble the message
                if buffer.len() == incoming_packet_info.total_parts as usize {
                    buffer.sort_by_key(|k| k.0);
                    let message: Vec<u8> = buffer.iter().flat_map(|(_, data)| data.clone()).collect();
                    
                    drop(buffer);

                    trace!(
                        "Reassembled message from session {}: {}",
                        incoming_packet_info.session_id,
                        String::from_utf8_lossy(&message)
                    );

                    function(message).await;

                    message_buffers.remove(&incoming_packet_info.session_id);

                    // Acknowledge the received chunk
                    let ack = format!("ACK {}", incoming_packet_info.session_id);
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

                if 
                    received_data[0] == ACK_BYTES[0] && 
                    received_data[1] == ACK_BYTES[1] &&
                    received_data[2] == ACK_BYTES[2] &&
                    received_data[3] == ACK_BYTES[3] {
                    return Vec::new();
                }
                
                let incoming_packet_info = IncomingPacketInfo::new(received_data);
                let payload = received_data[HEADER_SIZE..].to_vec();
    
                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    incoming_packet_info.sequence_number, incoming_packet_info.total_parts, incoming_packet_info.session_id, addr
                );
    
                // Add the chunk to the message buffer
                let mut buffer = message_buffers
                    .entry(incoming_packet_info.session_id.clone())
                    .or_insert_with(Vec::new);
    
                buffer.push((incoming_packet_info.sequence_number, payload));
    
                // If all chunks are received, reassemble the message
                if buffer.len() == incoming_packet_info.total_parts as usize {
                    buffer.sort_by_key(|k| k.0);
                    let message: Vec<u8> = buffer.iter().flat_map(|(_, data)| data.clone()).collect();
                    
                    drop(buffer);
    
                    trace!(
                        "Reassembled message from session {}: {}",
                        incoming_packet_info.session_id,
                        String::from_utf8_lossy(&message)
                    );
                    
                    message_buffers.remove(&incoming_packet_info.session_id);
    
                    // Acknowledge the received chunk
                    let ack = format!("ACK {}", incoming_packet_info.session_id);
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

                if 
                    received_data[0] == ACK_BYTES[0] && 
                    received_data[1] == ACK_BYTES[1] &&
                    received_data[2] == ACK_BYTES[2] &&
                    received_data[3] == ACK_BYTES[3] {
                    return;
                }

                let incoming_packet_info = IncomingPacketInfo::new(received_data);
                let payload = received_data[HEADER_SIZE..].to_vec();
    
                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    incoming_packet_info.sequence_number, incoming_packet_info.total_parts, incoming_packet_info.session_id, addr
                );
    
                // Add the chunk to the message buffer
                let mut buffer = message_buffers
                    .entry(incoming_packet_info.session_id.clone())
                    .or_insert_with(Vec::new);
    
                buffer.push((incoming_packet_info.sequence_number, payload));
    
                // If all chunks are received, reassemble the message
                if buffer.len() == incoming_packet_info.total_parts as usize {
                    buffer.sort_by_key(|k| k.0);
                    let message: Vec<u8> = buffer.iter().flat_map(|(_, data)| data.clone()).collect();
                    
                    drop(buffer);
    
                    trace!(
                        "Reassembled message from session {}: {}",
                        incoming_packet_info.session_id,
                        String::from_utf8_lossy(&message)
                    );

                    let result: Vec<u8> = function(message).await;

                    message_buffers.remove(&incoming_packet_info.session_id);

                    // Acknowledge the received chunk
                    let ack = format!("ACK {}", incoming_packet_info.session_id);
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

        let message_buffers: DashMap<u64, Vec<(u16, Vec<u8>)>, RandomState> = DashMap::default();
        let mut buf = vec![0u8; 1500];
        
        loop {
            if let Ok((len, addr)) = receiver.recv_from(&mut buf).await {
                let received_data = &buf[..len];

                if 
                    received_data[0] == ACK_BYTES[0] && 
                    received_data[1] == ACK_BYTES[1] &&
                    received_data[2] == ACK_BYTES[2] &&
                    received_data[3] == ACK_BYTES[3] {
                    return;
                }

                let incoming_packet_info = IncomingPacketInfo::new(received_data);
                let payload = received_data[HEADER_SIZE..].to_vec();
    
                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    incoming_packet_info.sequence_number, incoming_packet_info.total_parts, incoming_packet_info.session_id, addr
                );
    
                // Add the chunk to the message buffer
                let mut buffer = message_buffers
                    .entry(incoming_packet_info.session_id.clone())
                    .or_insert_with(Vec::new);
    
                buffer.push((incoming_packet_info.sequence_number, payload));
    
                // If all chunks are received, reassemble the message
                if buffer.len() == incoming_packet_info.total_parts as usize {
                    buffer.sort_by_key(|k| k.0);
                    let message: Vec<u8> = buffer.iter().flat_map(|(_, data)| data.clone()).collect();
                    
                    drop(buffer);
    
                    trace!(
                        "Reassembled message from session {}: {}",
                        incoming_packet_info.session_id,
                        String::from_utf8_lossy(&message)
                    );

                    function(message).await;

                    message_buffers.remove(&incoming_packet_info.session_id);

                    // Acknowledge the received chunk
                    let ack = format!("ACK {}", incoming_packet_info.session_id);
                    _ = receiver.send_to(ack.as_bytes(), addr).await;
                }
            }
        }
    }

    /// Internal use only
    pub(crate) async fn get_message_external(
        receiver: Arc<UdpSocket>, 
        buf: &mut [u8; 1500]) -> Vec<u8> {
        let message_buffers: DashMap<u64, Vec<(u16, Vec<u8>)>, RandomState> = DashMap::default();

        let message = loop {
            if let Ok((len, addr)) = receiver.recv_from(buf).await {
                let received_data = &buf[..len];
                
                if 
                    received_data[0] == ACK_BYTES[0] && 
                    received_data[1] == ACK_BYTES[1] &&
                    received_data[2] == ACK_BYTES[2] &&
                    received_data[3] == ACK_BYTES[3] {
                    return Vec::new();
                }

                let incoming_packet_info = IncomingPacketInfo::new(received_data);
                let payload = received_data[HEADER_SIZE..].to_vec();
    
                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    incoming_packet_info.sequence_number, incoming_packet_info.total_parts, incoming_packet_info.session_id, addr
                );
    
                // Add the chunk to the message buffer
                let mut buffer = message_buffers
                    .entry(incoming_packet_info.session_id.clone())
                    .or_insert_with(Vec::new);
    
                buffer.push((incoming_packet_info.sequence_number, payload));
    
                // If all chunks are received, reassemble the message
                if buffer.len() == incoming_packet_info.total_parts as usize {
                    buffer.sort_by_key(|k| k.0);
                    let message: Vec<u8> = buffer.iter().flat_map(|(_, data)| data.clone()).collect();
                    
                    drop(buffer);
    
                    trace!(
                        "Reassembled message from session {}: {}",
                        incoming_packet_info.session_id,
                        String::from_utf8_lossy(&message)
                    );
                    
                    message_buffers.remove(&incoming_packet_info.session_id);
    
                    // Acknowledge the received chunk
                    let ack = format!("ACK {}", incoming_packet_info.session_id);
                    _ = receiver.send_to(ack.as_bytes(), addr).await;
    
                    break message
                }
            }
        };

        message
    }
}
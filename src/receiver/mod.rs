use std::{future::Future, sync::Arc, u64};

use ahash::RandomState;
use helpers::{get_ack_message, IncomingPacketInfo};
use log::trace;
use moka::sync::{Cache, CacheBuilder};
use tokio::{io, net::UdpSocket, sync::RwLock};

use crate::{message_status::MessageStatus, sender::Sender, HEADER_SIZE};

pub(crate) mod helpers;

const ACK_BYTES: &[u8; 4]  = b"ACK ";

#[derive(Clone)]
pub struct Receiver {
    receiver: Arc<UdpSocket>,
    message_buffers: Cache<u64, Arc<RwLock<Vec<(u16, Vec<u8>)>>>, RandomState>,
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
            message_buffers: Cache::builder().build_with_hasher(RandomState::new()),
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
                    continue;
                }

                let incoming_packet_info = IncomingPacketInfo::new(received_data);
                let payload = received_data[HEADER_SIZE..].to_vec();

                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    incoming_packet_info.sequence_number, incoming_packet_info.total_parts, incoming_packet_info.session_id, addr
                );

                // Add the chunk to the message buffer
                let message_lock = if message_buffers.contains_key(&incoming_packet_info.session_id) {
                    message_buffers.get(&incoming_packet_info.session_id).unwrap()
                } else {
                    message_buffers.insert(incoming_packet_info.session_id, Arc::new(RwLock::new(Vec::default())));
                    message_buffers.get(&incoming_packet_info.session_id).unwrap()
                };

                let mut buffer = message_lock.write().await;
                buffer.push((incoming_packet_info.sequence_number, payload));
                let len = buffer.len();
                drop(buffer);

                // If all chunks are received, reassemble the message
                if len == incoming_packet_info.total_parts as usize {
                    let inner_message_lock = message_buffers.remove(&incoming_packet_info.session_id).unwrap();
                    let mut inner_message = inner_message_lock.write().await;
                    inner_message.sort_by_key(|k| k.0);
                    let message: Vec<u8> = inner_message.iter().flat_map(|(_, data)| data.clone()).collect();
                    drop(inner_message);
                    
                    trace!(
                        "Reassembled message from session {}: {}",
                        incoming_packet_info.session_id,
                        String::from_utf8_lossy(&message)
                    );

                    function(message).await;

                    // Acknowledge the received chunk
                    let ack_message: [u8; 12] = get_ack_message(incoming_packet_info.session_id);
                    _ = receiver.send_to(&ack_message, addr).await;
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
                let message_lock = if message_buffers.contains_key(&incoming_packet_info.session_id) {
                    message_buffers.get(&incoming_packet_info.session_id).unwrap()
                } else {
                    message_buffers.insert(incoming_packet_info.session_id, Arc::new(RwLock::new(Vec::default())));
                    message_buffers.get(&incoming_packet_info.session_id).unwrap()
                };

                let mut buffer = message_lock.write().await;
                buffer.push((incoming_packet_info.sequence_number, payload));
                let len = buffer.len();
                drop(buffer);

                // If all chunks are received, reassemble the message
                if len == incoming_packet_info.total_parts as usize {
                    let inner_message_lock = message_buffers.remove(&incoming_packet_info.session_id).unwrap();
                    let mut inner_message = inner_message_lock.write().await;
                    inner_message.sort_by_key(|k| k.0);
                    let message: Vec<u8> = inner_message.iter().flat_map(|(_, data)| data.clone()).collect();
                    drop(inner_message);
                    
                    trace!(
                        "Reassembled message from session {}: {}",
                        incoming_packet_info.session_id,
                        String::from_utf8_lossy(&message)
                    );
                    
                    // Acknowledge the received chunk
                    let ack_message: [u8; 12] = get_ack_message(incoming_packet_info.session_id);
                    _ = receiver.send_to(&ack_message, addr).await;
    
                    break message
                }
            } else {
                break Vec::new()
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
                    continue;
                }

                let incoming_packet_info = IncomingPacketInfo::new(received_data);
                let payload = received_data[HEADER_SIZE..].to_vec();
    
                trace!(
                    "Received chunk {}/{} from session {} from {}",
                    incoming_packet_info.sequence_number, incoming_packet_info.total_parts, incoming_packet_info.session_id, addr
                );
    
                // Add the chunk to the message buffer
                let message_lock = if message_buffers.contains_key(&incoming_packet_info.session_id) {
                    message_buffers.get(&incoming_packet_info.session_id).unwrap()
                } else {
                    message_buffers.insert(incoming_packet_info.session_id, Arc::new(RwLock::new(Vec::default())));
                    message_buffers.get(&incoming_packet_info.session_id).unwrap()
                };

                let mut buffer = message_lock.write().await;
                buffer.push((incoming_packet_info.sequence_number, payload));
                let len = buffer.len();
                drop(buffer);

                // If all chunks are received, reassemble the message
                if len == incoming_packet_info.total_parts as usize {
                    let inner_message_lock = message_buffers.remove(&incoming_packet_info.session_id).unwrap();
                    let mut inner_message = inner_message_lock.write().await;
                    inner_message.sort_by_key(|k| k.0);
                    let message: Vec<u8> = inner_message.iter().flat_map(|(_, data)| data.clone()).collect();
                    drop(inner_message);
                    
                    trace!(
                        "Reassembled message from session {}: {}",
                        incoming_packet_info.session_id,
                        String::from_utf8_lossy(&message)
                    );

                    let result: Vec<u8> = function(message).await;

                    // Acknowledge the received chunk
                    let ack_message: [u8; 12] = get_ack_message(incoming_packet_info.session_id);
                    _ = receiver.send_to(&ack_message, addr).await;

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

        let message_buffers: Cache<u64, Arc<RwLock<Vec<(u16, Vec<u8>)>>>, RandomState> = CacheBuilder::default().build_with_hasher(RandomState::new());
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
                let message_lock = if message_buffers.contains_key(&incoming_packet_info.session_id) {
                    message_buffers.get(&incoming_packet_info.session_id).unwrap()
                } else {
                    message_buffers.insert(incoming_packet_info.session_id, Arc::new(RwLock::new(Vec::default())));
                    message_buffers.get(&incoming_packet_info.session_id).unwrap()
                };

                let mut buffer = message_lock.write().await;
                buffer.push((incoming_packet_info.sequence_number, payload));
                let len = buffer.len();
                drop(buffer);

                // If all chunks are received, reassemble the message
                if len == incoming_packet_info.total_parts as usize {
                    let inner_message_lock = message_buffers.remove(&incoming_packet_info.session_id).unwrap();
                    let mut inner_message = inner_message_lock.write().await;
                    inner_message.sort_by_key(|k| k.0);
                    let message: Vec<u8> = inner_message.iter().flat_map(|(_, data)| data.clone()).collect();
                    drop(inner_message);

                    trace!(
                        "Reassembled message from session {}: {}",
                        incoming_packet_info.session_id,
                        String::from_utf8_lossy(&message)
                    );

                    function(message).await;

                    // Acknowledge the received chunk
                    let ack_message: [u8; 12] = get_ack_message(incoming_packet_info.session_id);
                    _ = receiver.send_to(&ack_message, addr).await;
                    break;
                }
            }
        }
    }

    /// Internal use only
    pub(crate) async fn get_message_external(
        receiver: Arc<UdpSocket>, 
        buf: &mut [u8; 1500]) -> Vec<u8> {
        let message_buffers: Cache<u64, Arc<RwLock<Vec<(u16, Vec<u8>)>>>, RandomState> = CacheBuilder::default().build_with_hasher(RandomState::new());

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
                let message_lock = if message_buffers.contains_key(&incoming_packet_info.session_id) {
                    message_buffers.get(&incoming_packet_info.session_id).unwrap()
                } else {
                    message_buffers.insert(incoming_packet_info.session_id, Arc::new(RwLock::new(Vec::default())));
                    message_buffers.get(&incoming_packet_info.session_id).unwrap()
                };

                let mut buffer = message_lock.write().await;
                buffer.push((incoming_packet_info.sequence_number, payload));
                let len = buffer.len();
                drop(buffer);

                // If all chunks are received, reassemble the message
                if len == incoming_packet_info.total_parts as usize {
                    let inner_message_lock = message_buffers.remove(&incoming_packet_info.session_id).unwrap();
                    let mut inner_message = inner_message_lock.write().await;
                    inner_message.sort_by_key(|k| k.0);
                    let message: Vec<u8> = inner_message.iter().flat_map(|(_, data)| data.clone()).collect();
                    drop(inner_message);

                    trace!(
                        "Reassembled message from session {}: {}",
                        incoming_packet_info.session_id,
                        String::from_utf8_lossy(&message)
                    );
                    
                    // Acknowledge the received chunk
                    let ack_message: [u8; 12] = get_ack_message(incoming_packet_info.session_id);
                    _ = receiver.send_to(&ack_message, addr).await;
    
                    break message
                }
            } else {
                break Vec::new()
            }
        };

        message
    }
}
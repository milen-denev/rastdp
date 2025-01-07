use log::trace;
use rand::Rng;

use crate::message_status::MessageStatus;

use super::{MESSAGE_SIZE, X32_CHECKSUM};

pub(crate) struct PacketInfo {
    message_len: usize,
    pub(crate) session_id: u64,
    compressed: bool,
    message_status: MessageStatus
}

impl PacketInfo {
    pub(crate) fn new(message_len: usize, message_status: MessageStatus, compress: bool) -> Self {
        let session_id = rand::thread_rng().gen_range(0..u64::MAX);

        PacketInfo {
            session_id: session_id,
            message_len: message_len,
            compressed: compress,
            message_status: message_status
        }
    }

    pub(crate) fn get_total_parts(&self) -> [u8; 2] {
        let total_chunks = (self.message_len + MESSAGE_SIZE as usize - 1) / MESSAGE_SIZE as usize;
        let total_parts = (total_chunks as u16).to_be_bytes();
        total_parts
    }

    pub(crate) fn get_session_id(&self) -> [u8; 8] {
        let session_id_bytes = self.session_id.to_be_bytes();
        session_id_bytes
    }

    pub(crate) fn get_message_status(&self) -> [u8; 1] {
        let message_status = [self.message_status.to_u8()];
        message_status
    }

    pub(crate) fn get_compressed(&self) -> [u8; 1] {
        let message_status = [self.compressed as u8];
        message_status
    }
}

pub(crate) fn construct_slice(chunk: &[u8], packet_info: &PacketInfo, i: usize) -> Vec<u8> {
    let sequence_number = (i as u16 + 1).to_be_bytes();

    let mut packet = Vec::with_capacity(chunk.len() + 8 + 2 + 2 + 1 + 1 + 4);

    packet.extend_from_slice(&sequence_number);
    packet.extend_from_slice(&packet_info.get_total_parts());
    packet.extend_from_slice(&packet_info.get_session_id());

    packet.extend_from_slice(&packet_info.get_message_status());
    packet.extend_from_slice(&packet_info.get_compressed());

    let checksum = X32_CHECKSUM.checksum(&chunk).to_be_bytes();

    packet.extend_from_slice(&checksum);

    packet.extend_from_slice(chunk);

    trace!(
        "Sent chunk {}",
        i + 1
    );

    packet
}
use crate::message_status::MessageStatus;

use super::HEADER_SIZE;

pub(crate) struct IncomingPacketInfo {
    pub(crate) sequence_number: u16,
    pub(crate) total_parts: u16,
    pub(crate) session_id: u64,
    pub(crate) message_status: MessageStatus,
    pub(crate) compressed: bool, 
    pub(crate) checksum: u32
}

impl IncomingPacketInfo {
    pub(crate) fn new(received_data: &[u8]) -> Self {
        let header = &received_data[..HEADER_SIZE];
        
        let sequence_number = u16::from_be_bytes([header[0], header[1]]);
        let total_parts = u16::from_be_bytes([header[2], header[3]]);
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

        let message_status = header[12];
        let compressed = header[13];
        let checksum = u32::from_be_bytes(
        [
            header[14], 
            header[15], 
            header[16],
            header[17],
        ]);

        IncomingPacketInfo {
            sequence_number: sequence_number,
            total_parts: total_parts,
            session_id: session_id,
            message_status: MessageStatus::from_u8(message_status),
            compressed: if compressed == 0 { false } else if compressed == 1{ true } else { panic!() },
            checksum: checksum
        }
    }
}
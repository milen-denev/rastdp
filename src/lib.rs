pub mod receiver;
pub mod sender;
pub mod message_status;

// Session Id (8 bytes)
// Sequence Number (2 bytes)
// Total Parts (2 bytes)
// Protocol Status (1 byte) 
// Compress (1 byte)
// Checksum (4 bytes)
pub(crate) const MESSAGE_SIZE: u16 = 1482;
pub(crate) const HEADER_SIZE: usize = 1500 - MESSAGE_SIZE as usize;
pub(crate) const X32_CHECKSUM: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);
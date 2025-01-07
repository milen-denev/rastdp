/// ```rust
/// static RECEIVER: async_lazy::Lazy<Arc<Receiver>> = async_lazy::Lazy::const_new(|| Box::pin(async {
///     let receiver = Receiver::new("127.0.0.1:8080").await.unwrap();
///     Arc::new(receiver)
/// }));
/// 
/// #[tokio::main]
/// async fn main() -> io::Result<()> {
///     // Server processing requests
///     tokio::spawn(async { 
///         let receiver = RECEIVER.force().await;
///         let receiver_clone = receiver.clone();
/// 
///         // Function that returns a result
///         receiver_clone.start_processing_function_result(
///             a_function_receiving_request_and_returning_result
///         ).await;
/// 
///         // Function that only processes requests
///         receiver.start_processing_function(
///             a_function_receiving_request
///         ).await;
///     });
///     loop {
///         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
///     }
/// }
/// 
/// async fn a_function_receiving_request_and_returning_result(buf: Vec<u8>) -> Vec<u8> {
///     println!("buf: {:?}", buf.len());
///     vec![1, 2, 3, 4, 5]
/// }
/// 
/// async fn a_function_receiving_request(buf: Vec<u8>) {
///     println!("buf: {:?}", buf.len());
/// }
/// ```
pub mod receiver;

/// ```rust
/// static SENDER: async_lazy::Lazy<Arc<Sender>> = async_lazy::Lazy::const_new(|| Box::pin(async {
///     let sender = Sender::new("127.0.0.1:8080".to_string()).await.unwrap();
///     Arc::new(sender)
/// }));
/// 
/// #[tokio::main]
/// async fn main() -> io::Result<()> {
///     let sender = SENDER.force().await;
///     for i in 0..1_000_000 {
///         tokio::spawn(async {
///             let sender_clone = sender.clone(); 
///             let test = sender_clone.send_message_get_reply("Hello, world!".repeat(10).as_bytes()).await; 
///             println!("test: {:?}", test);
///         });
///     }
///     
///     loop {
///         tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
///     }
/// }
/// ```
pub mod sender;

pub(crate) mod message_status;

// Session Id (8 bytes)
// Sequence Number (2 bytes)
// Total Parts (2 bytes)
// Protocol Status (1 byte) 
// Compress (1 byte)
// Checksum (4 bytes)
#[cfg(feature = "enable_checksums")]
pub(crate) const MESSAGE_SIZE: u16 = 1482;

#[cfg(not(feature = "enable_checksums"))]
pub(crate) const MESSAGE_SIZE: u16 = 1486;

pub(crate) const HEADER_SIZE: usize = 1500 - MESSAGE_SIZE as usize;

#[cfg(feature = "enable_checksums")]
pub(crate) const X32_CHECKSUM: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_CKSUM);
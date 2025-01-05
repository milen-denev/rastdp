//use rastdp::{receiver::Receiver, sender::Sender};
use tokio::io;
//use std::sync::Arc;

// static RECEIVER: async_lazy::Lazy<Arc<Receiver>> = async_lazy::Lazy::const_new(|| Box::pin(async {
//     let receiver = Receiver::new("127.0.0.1:8080").await.unwrap();
//     Arc::new(receiver)
// }));

// static SENDER: async_lazy::Lazy<Arc<Sender>> = async_lazy::Lazy::const_new(|| Box::pin(async {
//     let sender = Sender::new("127.0.0.1:8080".to_string()).await.unwrap();
//     Arc::new(sender)
// }));

#[tokio::main]
async fn main() -> io::Result<()> {
    // Server processing requests
    // tokio::spawn(async { 
    //     let receiver = RECEIVER.force().await;
    //     let receiver_clone = receiver.clone();

    //     // Function that returns a result
    //     receiver_clone.start_processing_function_result(
    //         a_function_receiving_request_and_returning_result
    //     ).await;

    //     // Function that only processes requests
    //     receiver.start_processing_function(
    //         a_function_receiving_request
    //     ).await;
    // });

    // Client sending requests
    // let sender = SENDER.force().await;
    // for i in 0..1_000_000 {

    //     tokio::spawn(async {
    //         let sender_clone = sender.clone(); 
    //         let test = sender_clone.send_message_get_reply("Hello, world!".repeat(10).as_bytes()).await; 
    //         println!("test: {:?}", test);
    //     });
    // }

    // loop {
    //     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    // }

    Ok(())
}

// async fn a_function_receiving_request_and_returning_result(buf: Vec<u8>) -> Vec<u8> {
//     println!("buf: {:?}", buf.len());
//     vec![1, 2, 3, 4, 5]
// }

// async fn a_function_receiving_request(buf: Vec<u8>) {
//     println!("buf: {:?}", buf.len());
// }
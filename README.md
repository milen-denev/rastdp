# Rastdp (Rasterized Datagram Protocol)

## A UDP-based asynchronous communication protocol optimized for efficient inter-application messaging.

### Use cases

#### Key Features

- **Scalability**: Supports an infinite number of requests on both the server and client sides, ensuring robust handling of high-demand scenarios.
- **Efficiency**: Designed for low-latency, high-performance communication with minimal overhead.

#### Use Cases

Rastdp is ideal for low-latency networking environments, enabling rapid data exchange and seamless communication in scenarios where performance and scalability are critica

#### Coming Soon Features

- Compression: Reduce data size to further optimize bandwidth usage.
- Secure Communication: Enable encrypted network traffic for enhanced security and privacy.

#### Network Byte Layout

The network byte layout is currently not stable and is subject to change. A detailed paper on the finalized layout will be published to allow third-party integration.

#### Is it stable?

Short answer, no. It will get through many refinements and even changes to the table files. Until version 1.0.0 use it on your own risk.

### How to use the current API?

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
async-lazy = { version = "0.1.0", features = ["parking_lot"] }
```

```rust
use rastdp::{receiver::Receiver, sender::Sender};
use tokio::io;
use std::sync::Arc;

//Static receiver
static RECEIVER: async_lazy::Lazy<Arc<Receiver>> = async_lazy::Lazy::const_new(|| Box::pin(async {
    let receiver = Receiver::new("127.0.0.1:8080").await.unwrap();
    Arc::new(receiver)
}));

static SENDER: async_lazy::Lazy<Arc<Sender>> = async_lazy::Lazy::const_new(|| Box::pin(async {
    let sender = Sender::new("127.0.0.1:8080".to_string()).await.unwrap();
    Arc::new(sender)
}));

#[tokio::main]
async fn main() -> io::Result<()> {
    // Server processing requests
    tokio::spawn(async { 
        let receiver = RECEIVER.force().await;
        let receiver_clone = receiver.clone();

        // Function that returns a result
        receiver_clone.start_processing_function_result(
            a_function_receiving_request_and_returning_result
        ).await;

        // Function that only processes requests
        receiver.start_processing_function(
            a_function_receiving_request
        ).await;
    });

    // Client sending requests
    let sender = SENDER.force().await;
    for i in 0..1_000_000 {

        tokio::spawn(async {
            let sender_clone = sender.clone(); 
            let test = sender_clone.send_message_get_reply("Hello, world!".repeat(10).as_bytes()).await; 
            println!("test: {:?}", test);
        });
    }

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

async fn a_function_receiving_request_and_returning_result(buf: Vec<u8>) -> Vec<u8> {
    println!("buf: {:?}", buf.len());
    vec![1, 2, 3, 4, 5]
}

async fn a_function_receiving_request(buf: Vec<u8>) {
    println!("buf: {:?}", buf.len());
}
```

### License
Everything in this directory is distributed under GNU GENERAL PUBLIC LICENSE version 3.
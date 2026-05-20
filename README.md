# Async WebSocket

A convenience library for using websockets both in native and WASM environments!

```rust,no_run
use std::time::Duration;

use async_wsocket::prelude::*;
use futures_util::{SinkExt, StreamExt};

const NONCE: u64 = 123456789;

#[tokio::main]
async fn main() {
    let url =
        Url::parse("wss://relay.damus.io").unwrap();
    let mut socket: WebSocket =
        WebSocket::connect(&url, &ConnectionMode::direct())
            .await
            .unwrap();

    // Split sink and stream
    // let (mut tx, mut rx) = socket.split();

    // Send ping
    let nonce = NONCE.to_be_bytes().to_vec();
    socket.send(Message::Ping(nonce.clone())).await.unwrap();

    // Listen for messages
    while let Some(msg) = socket.next().await {
        if let Ok(Message::Pong(bytes)) = msg {
            assert_eq!(nonce, bytes);
            println!("Pong match!");
            break;
        }
    }
}
```

## Crate Feature Flags

The following crate feature flags are available:

| Feature                    | Default | Description                                                |
|----------------------------|:-------:|------------------------------------------------------------|
| `aws_lc_rs`                |   No    | Enable the Rustls `aws_lc_rs` crypto provider              |
| `native-tls`               |   No    | Enable native TLS support                                  |
| `native-tls-vendored`      |   No    | Enable vendored native TLS support                         |
| `ring`                     |   Yes   | Enable the Rustls `ring` crypto provider                   |
| `rustls-tls-native-roots`  |   No    | Enable Rustls TLS support with native root certificates    |
| `rustls-tls-webpki-roots`  |   Yes   | Enable Rustls TLS support with `webpki-roots` certificates |
| `socks`                    |   No    | Enable `socks` proxy support                               |

## Minimum Supported Rust Version (MSRV)

The MSRV for this project when compiled with `default` features and on `native` targets is `1.63.0`. 
MSRV for WASM targets is `1.73.0`

## License

This project is distributed under the MIT software license - see the [LICENSE](LICENSE) file for details

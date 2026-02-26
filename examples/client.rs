// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use async_wsocket::prelude::*;
use futures_util::{SinkExt, StreamExt};

const NONCE: u64 = 123456789;

#[tokio::main]
async fn main() {
    let url = Url::parse("wss://relay.damus.io").unwrap();
    let mut socket: WebSocket = WebSocket::connect(&url, &ConnectionMode::direct())
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

// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::time::Duration;

use async_wsocket::prelude::*;
use futures_util::{SinkExt, StreamExt};

const NONCE: u64 = 123456789;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let url =
        Url::parse("ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion").unwrap();
    let (mut tx, mut rx) =
        async_wsocket::connect(&url, &ConnectionMode::tor(), Duration::from_secs(120))
            .await
            .unwrap();

    // Send ping
    let nonce = NONCE.to_be_bytes().to_vec();
    tx.send(WsMessage::Ping(nonce.clone())).await.unwrap();

    // Listen for messages
    while let Some(msg) = rx.next().await {
        if let Ok(WsMessage::Pong(bytes)) = msg {
            assert_eq!(nonce, bytes);
            println!("Pong match!");
            break;
        }
    }
}

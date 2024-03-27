// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! WASM

use std::time::Duration;

use async_utility::time;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use thiserror::Error;
use url::Url;
use wasm_ws::{WebSocket, WsErr, WsMessage, WsStream};

pub type Sink = SplitSink<WsStream, WsMessage>;
pub type Stream = SplitStream<WsStream>;

#[derive(Debug, Error)]
pub enum Error {
    /// Ws error
    #[error("ws error: {0}")]
    Ws(#[from] WsErr),
    /// Timeout
    #[error("timeout")]
    Timeout,
}

pub async fn connect(url: &Url, timeout: Option<Duration>) -> Result<(Sink, Stream), Error> {
    let timeout = timeout.unwrap_or(Duration::from_secs(60));
    let (_ws, stream) = time::timeout(Some(timeout), WebSocket::connect(url))
        .await
        .ok_or(Error::Timeout)??;
    Ok(stream.split())
}

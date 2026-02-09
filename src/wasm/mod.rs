// Copyright (c) 2019-2022 Naja Melan
// Copyright (c) 2023-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Wasm

#![allow(clippy::arc_with_non_send_sync)]

use url::Url;
use wasm_bindgen_futures::spawn_local;

mod error;
mod event;
mod message;
mod pharos;
mod socket;
mod state;
mod stream;

pub use self::error::Error;
use self::event::{CloseEvent, WsEvent};
use self::pharos::SharedPharos;
use self::socket::WebSocket as WasmWebSocket;
use self::state::WsState;
pub(crate) use self::stream::WsStream;
use crate::socket::WebSocket;

pub async fn connect(url: &Url) -> Result<WebSocket, Error> {
    let (_ws, stream) = WasmWebSocket::connect(url).await?;
    Ok(WebSocket::wasm(stream))
}

/// Helper function to reduce code bloat
pub(crate) fn notify(pharos: SharedPharos<WsEvent>, evt: WsEvent) {
    spawn_local(async move {
        pharos
            .notify(evt)
            .await
            .map_err(|e| unreachable!("{:?}", e))
            .unwrap(); // only happens if we closed it.
    });
}

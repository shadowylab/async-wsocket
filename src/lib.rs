// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Async WebSocket

#![forbid(unsafe_code)]

use std::net::SocketAddr;
use std::time::Duration;

pub use futures_util;
pub use url::{self, Url};
#[cfg(target_arch = "wasm32")]
pub use wasm_ws::WsMessage;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::{Error, Message as WsMessage, Sink, Stream};
#[cfg(target_arch = "wasm32")]
pub use self::wasm::{Error, Sink, Stream};

/// Connect
///
/// **Proxy is ignored for WASM targets!**
pub async fn connect(
    url: &Url,
    _proxy: Option<SocketAddr>,
    timeout: Option<Duration>,
) -> Result<(Sink, Stream), Error> {
    #[cfg(not(target_arch = "wasm32"))]
    let (tx, rx) = self::native::connect(url, _proxy, timeout).await?;

    #[cfg(target_arch = "wasm32")]
    let (tx, rx) = self::wasm::connect(url, timeout).await?;

    Ok((tx, rx))
}

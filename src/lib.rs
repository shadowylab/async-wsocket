// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Async WebSocket

#![forbid(unsafe_code)]
#![warn(clippy::large_futures)]
#![cfg_attr(feature = "default", doc = include_str!("../README.md"))]

#[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
use std::net::SocketAddr;
use std::time::Duration;

pub use futures_util;
pub use url::{self, Url};

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::{Error, Message as WsMessage, Sink, Stream};
#[cfg(target_arch = "wasm32")]
pub use self::wasm::{Error, Sink, Stream, WsMessage};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConnectionMode {
    /// Direct
    #[default]
    Direct,
    /// Custom proxy
    #[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
    Proxy(SocketAddr),
    /// Embedded tor client
    #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
    Tor,
}

/// Connect
///
/// **Proxy is ignored for WASM targets!**
pub async fn connect(
    url: &Url,
    _mode: ConnectionMode,
    timeout: Duration,
) -> Result<(Sink, Stream), Error> {
    #[cfg(not(target_arch = "wasm32"))]
    let (tx, rx) = self::native::connect(url, _mode, timeout).await?;

    #[cfg(target_arch = "wasm32")]
    let (tx, rx) = self::wasm::connect(url, timeout).await?;

    Ok((tx, rx))
}

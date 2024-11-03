// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Async WebSocket

#![forbid(unsafe_code)]
#![warn(clippy::large_futures)]
#![cfg_attr(feature = "default", doc = include_str!("../README.md"))]

#[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
use std::net::SocketAddr;
#[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
use std::path::PathBuf;
use std::time::Duration;

pub use futures_util;
pub use url::{self, Url};

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
pub mod prelude;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::{Error, Message as WsMessage, Sink, Stream};
#[cfg(target_arch = "wasm32")]
pub use self::wasm::{Error, Sink, Stream, WsMessage};

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConnectionMode {
    /// Direct
    #[default]
    Direct,
    /// Custom proxy
    #[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
    Proxy(SocketAddr),
    /// Embedded tor client
    #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
    Tor {
        /// Path for cache and state data
        ///
        /// Mandatory for `android` and `ios` targets!
        custom_path: Option<PathBuf>,
    },
}

impl ConnectionMode {
    /// Direct connection
    #[inline]
    pub fn direct() -> Self {
        Self::Direct
    }

    /// Proxy
    #[inline]
    #[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
    pub fn proxy(addr: SocketAddr) -> Self {
        Self::Proxy(addr)
    }

    /// Embedded tor client
    #[inline]
    #[cfg(all(
        feature = "tor",
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        not(target_os = "ios"),
    ))]
    pub fn tor() -> Self {
        Self::Tor { custom_path: None }
    }

    /// Embedded tor client
    #[inline]
    #[cfg(all(
        feature = "tor",
        not(target_arch = "wasm32"),
        any(target_os = "android", target_os = "ios")
    ))]
    pub fn tor(data_path: PathBuf) -> Self {
        Self::Tor {
            custom_path: Some(data_path),
        }
    }
}

/// Connect
pub async fn connect(
    url: &Url,
    _mode: &ConnectionMode,
    timeout: Duration,
) -> Result<(Sink, Stream), Error> {
    #[cfg(not(target_arch = "wasm32"))]
    let (tx, rx) = self::native::connect(url, _mode, timeout).await?;

    #[cfg(target_arch = "wasm32")]
    let (tx, rx) = self::wasm::connect(url, timeout).await?;

    Ok((tx, rx))
}

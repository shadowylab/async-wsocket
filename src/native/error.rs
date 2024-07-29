// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as WsError;
use url::ParseError;

#[cfg(feature = "tor")]
use super::tor;

#[derive(Debug, Error)]
pub enum Error {
    /// I/O error
    #[error(transparent)]
    IO(#[from] std::io::Error),
    /// Ws error
    #[error(transparent)]
    Ws(#[from] WsError),
    /// Socks error
    #[cfg(feature = "socks")]
    #[error(transparent)]
    Socks(#[from] tokio_socks::Error),
    /// Tor error
    #[cfg(feature = "tor")]
    #[error(transparent)]
    Tor(#[from] tor::Error),
    /// Url parse error
    #[error(transparent)]
    Url(#[from] ParseError),
    /// Timeout
    #[error("timeout")]
    Timeout,
    /// Invalid DNS name
    #[error("invalid DNS name")]
    InvalidDNSName,
}

impl Error {
    #[inline]
    #[cfg(any(feature = "socks", feature = "tor"))]
    pub(super) fn empty_host() -> Self {
        Self::Url(ParseError::EmptyHost)
    }

    #[inline]
    #[cfg(any(feature = "socks", feature = "tor"))]
    pub(super) fn invalid_port() -> Self {
        Self::Url(ParseError::InvalidPort)
    }
}

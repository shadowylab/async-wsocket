// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Native

#[cfg(feature = "socks")]
use std::net::SocketAddr;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::protocol::Role;
pub use tokio_tungstenite::tungstenite::Message;
pub use tokio_tungstenite::WebSocketStream;
use url::Url;

mod error;
#[cfg(feature = "socks")]
mod socks;

pub use self::error::Error;
#[cfg(feature = "socks")]
use self::socks::TcpSocks5Stream;
use crate::socket::WebSocket;
use crate::ConnectionMode;

pub async fn connect(url: &Url, mode: &ConnectionMode) -> Result<WebSocket, Error> {
    match mode {
        ConnectionMode::Direct => connect_direct(url).await,
        #[cfg(feature = "socks")]
        ConnectionMode::Proxy(proxy) => connect_proxy(url, *proxy).await,
    }
}

async fn connect_direct(url: &Url) -> Result<WebSocket, Error> {
    let host: &str = url.host_str().ok_or_else(Error::empty_host)?;
    let port: u16 = url
        .port_or_known_default()
        .ok_or_else(Error::invalid_port)?;

    let host: String = format!("{}:{}", host, port);

    let tcp_stream: TcpStream = tokio_happy_eyeballs::connect(host).await?;

    // NOT REMOVE `Box::pin`!
    // Use `Box::pin` to fix stack overflow on windows targets due to large `Future`
    let (stream, _) = Box::pin(tokio_tungstenite::client_async_tls(
        url.as_str(),
        tcp_stream,
    ))
    .await?;
    Ok(WebSocket::tokio(Box::new(stream)))
}

#[cfg(feature = "socks")]
async fn connect_proxy(url: &Url, proxy: SocketAddr) -> Result<WebSocket, Error> {
    let host: &str = url.host_str().ok_or_else(Error::empty_host)?;
    let port: u16 = url
        .port_or_known_default()
        .ok_or_else(Error::invalid_port)?;
    let addr: String = format!("{host}:{port}");

    let conn: TcpStream = TcpSocks5Stream::connect(proxy, addr).await?;
    // NOT REMOVE `Box::pin`!
    // Use `Box::pin` to fix stack overflow on windows targets due to large `Future`
    let (stream, _) = Box::pin(tokio_tungstenite::client_async_tls(url.as_str(), conn)).await?;
    Ok(WebSocket::tokio(Box::new(stream)))
}

#[inline]
pub async fn accept<S>(raw_stream: S) -> Result<WebSocketStream<S>, Error>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    Ok(tokio_tungstenite::accept_async(raw_stream).await?)
}

/// Take an already upgraded websocket connection
///
/// Useful for when using [hyper] or [warp] or any other HTTP server
#[inline]
pub async fn take_upgraded<S>(raw_stream: S) -> WebSocketStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    WebSocketStream::from_raw_socket(raw_stream, Role::Server, None).await
}

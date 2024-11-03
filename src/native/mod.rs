// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Native

#[cfg(feature = "socks")]
use std::net::SocketAddr;
#[cfg(feature = "tor")]
use std::path::PathBuf;
use std::time::Duration;

#[cfg(feature = "tor")]
use arti_client::DataStream;
use async_utility::time;
use futures_util::StreamExt;
use tokio::io::{AsyncRead, AsyncWrite};
#[cfg(feature = "socks")]
use tokio::net::TcpStream;
pub use tokio_tungstenite::tungstenite::Message;
pub use tokio_tungstenite::WebSocketStream;
use url::Url;

mod error;
#[cfg(feature = "socks")]
mod socks;
mod stream;
#[cfg(feature = "tor")]
pub mod tor;

pub use self::error::Error;
#[cfg(feature = "socks")]
use self::socks::TcpSocks5Stream;
use self::stream::WebSocket;
pub use self::stream::{Sink, Stream};
use crate::ConnectionMode;

pub async fn connect(
    url: &Url,
    mode: &ConnectionMode,
    timeout: Duration,
) -> Result<(Sink, Stream), Error> {
    let stream: WebSocket = match mode {
        ConnectionMode::Direct => connect_direct(url, timeout).await?,
        #[cfg(feature = "socks")]
        ConnectionMode::Proxy(proxy) => connect_proxy(url, *proxy, timeout).await?,
        #[cfg(feature = "tor")]
        ConnectionMode::Tor { custom_path } => {
            connect_tor(url, timeout, custom_path.as_ref()).await?
        }
    };

    match stream {
        WebSocket::Std(stream) => {
            let (tx, rx) = stream.split();
            Ok((Sink::Std(tx), Stream::Std(rx)))
        }
        #[cfg(feature = "tor")]
        WebSocket::Tor(stream) => {
            let (tx, rx) = stream.split();
            Ok((Sink::Tor(tx), Stream::Tor(rx)))
        }
    }
}

async fn connect_direct(url: &Url, timeout: Duration) -> Result<WebSocket, Error> {
    // NOT REMOVE `Box::pin`!
    // Use `Box::pin` to fix stack overflow on windows targets due to large `Future`
    let (stream, _) = Box::pin(time::timeout(
        Some(timeout),
        tokio_tungstenite::connect_async(url.as_str()),
    ))
    .await
    .ok_or(Error::Timeout)??;
    Ok(WebSocket::Std(stream))
}

#[cfg(feature = "socks")]
async fn connect_proxy(
    url: &Url,
    proxy: SocketAddr,
    timeout: Duration,
) -> Result<WebSocket, Error> {
    let host: &str = url.host_str().ok_or_else(Error::empty_host)?;
    let port: u16 = url
        .port_or_known_default()
        .ok_or_else(Error::invalid_port)?;
    let addr: String = format!("{host}:{port}");

    let conn: TcpStream = TcpSocks5Stream::connect(proxy, addr).await?;
    // NOT REMOVE `Box::pin`!
    // Use `Box::pin` to fix stack overflow on windows targets due to large `Future`
    let (stream, _) = Box::pin(time::timeout(
        Some(timeout),
        tokio_tungstenite::client_async_tls(url.as_str(), conn),
    ))
    .await
    .ok_or(Error::Timeout)??;
    Ok(WebSocket::Std(stream))
}

#[cfg(feature = "tor")]
async fn connect_tor(
    url: &Url,
    timeout: Duration,
    custom_path: Option<&PathBuf>,
) -> Result<WebSocket, Error> {
    let host: &str = url.host_str().ok_or_else(Error::empty_host)?;
    let port: u16 = url
        .port_or_known_default()
        .ok_or_else(Error::invalid_port)?;

    let conn: DataStream = tor::connect(host, port, custom_path).await?;
    // NOT REMOVE `Box::pin`!
    // Use `Box::pin` to fix stack overflow on windows targets due to large `Future`
    let (stream, _) = Box::pin(time::timeout(
        Some(timeout),
        tokio_tungstenite::client_async_tls(url.as_str(), conn),
    ))
    .await
    .ok_or(Error::Timeout)??;
    Ok(WebSocket::Tor(stream))
}

#[inline]
pub async fn accept<S>(raw_stream: S) -> Result<WebSocketStream<S>, Error>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    Ok(tokio_tungstenite::accept_async(raw_stream).await?)
}

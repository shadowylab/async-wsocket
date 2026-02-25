// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Native

use std::net::SocketAddr;
#[cfg(feature = "tor")]
use std::path::PathBuf;
use std::time::Duration;

#[cfg(feature = "tor")]
use arti_client::DataStream;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::time;
use tokio_tungstenite::tungstenite::protocol::Role;
pub use tokio_tungstenite::tungstenite::Message;
pub use tokio_tungstenite::WebSocketStream;
use url::Url;

mod error;
#[cfg(feature = "socks")]
mod socks;
#[cfg(feature = "tor")]
pub mod tor;

pub use self::error::Error;
#[cfg(feature = "socks")]
use self::socks::TcpSocks5Stream;
use crate::socket::WebSocket;
use crate::ConnectionMode;

pub async fn connect(
    url: &Url,
    mode: &ConnectionMode,
    timeout: Duration,
) -> Result<WebSocket, Error> {
    match mode {
        ConnectionMode::Direct => connect_direct(url, timeout).await,
        #[cfg(feature = "socks")]
        ConnectionMode::Proxy(proxy) => connect_proxy(url, *proxy, timeout).await,
        #[cfg(feature = "tor")]
        ConnectionMode::Tor { custom_path } => {
            connect_tor(url, timeout, custom_path.as_ref()).await
        }
    }
}

/// Happy Eyeballs connection delay (RFC 8305).
const HAPPY_EYEBALLS_DELAY: Duration = Duration::from_millis(250);

async fn connect_direct(url: &Url, timeout: Duration) -> Result<WebSocket, Error> {
    let host: &str = url.host_str().ok_or_else(Error::empty_host)?;
    let port: u16 = url
        .port_or_known_default()
        .ok_or_else(Error::invalid_port)?;

    let conn_fut = async {
        let tcp_stream = happy_eyeballs_connect(host, port).await?;

        // NOT REMOVE `Box::pin`!
        // Use `Box::pin` to fix stack overflow on windows targets due to large `Future`
        let (stream, _) = Box::pin(tokio_tungstenite::client_async_tls(
            url.as_str(),
            tcp_stream,
        ))
        .await?;

        Ok::<_, Error>(stream)
    };

    let stream = time::timeout(timeout, conn_fut)
        .await
        .map_err(|_| Error::Timeout)??;

    Ok(WebSocket::Tokio(stream))
}

/// Connect to a host using the Happy Eyeballs algorithm (RFC 8305).
///
/// When DNS returns both IPv6 and IPv4 addresses, tries the preferred family
/// first and starts the other family after a 250ms delay if the first hasn't
/// connected yet. Uses whichever connection succeeds first.
async fn happy_eyeballs_connect(host: &str, port: u16) -> Result<TcpStream, Error> {
    let addrs: Vec<SocketAddr> = tokio::net::lookup_host(format!("{host}:{port}"))
        .await?
        .collect();

    if addrs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AddrNotAvailable,
            "DNS resolution returned no addresses",
        )
        .into());
    }

    // Separate into IPv6 and IPv4, preserving order within each group
    let mut ipv6: Vec<SocketAddr> = Vec::new();
    let mut ipv4: Vec<SocketAddr> = Vec::new();
    for addr in addrs {
        if addr.is_ipv6() {
            ipv6.push(addr);
        } else {
            ipv4.push(addr);
        }
    }

    // If only one family, try addresses sequentially
    if ipv4.is_empty() {
        return try_addrs_sequential(&ipv6).await;
    }
    if ipv6.is_empty() {
        return try_addrs_sequential(&ipv4).await;
    }

    // Both families available: Happy Eyeballs
    // Try first IPv6 address, after delay start first IPv4 in parallel
    let ipv6_first = ipv6[0];
    let ipv4_first = ipv4[0];

    // Pin the IPv6 future so it survives across select boundaries
    let ipv6_fut = TcpStream::connect(ipv6_first);
    tokio::pin!(ipv6_fut);

    // Phase 1: Give IPv6 a 250ms head start
    tokio::select! {
        result = &mut ipv6_fut => {
            match result {
                Ok(stream) => return Ok(stream),
                // IPv6 failed fast, try IPv4 directly
                Err(_) => return try_addrs_sequential(&ipv4).await,
            }
        }
        _ = tokio::time::sleep(HAPPY_EYEBALLS_DELAY) => {
            // Timer fired, IPv6 still pending. Start IPv4 and race both.
        }
    }

    // Phase 2: Race the still-pending IPv6 against a new IPv4 attempt.
    // Use a loop so that if one fails, we keep waiting for the other.
    let ipv4_fut = TcpStream::connect(ipv4_first);
    tokio::pin!(ipv4_fut);

    let mut ipv6_done = false;
    let mut ipv4_done = false;

    loop {
        tokio::select! {
            result = &mut ipv6_fut, if !ipv6_done => {
                match result {
                    Ok(stream) => return Ok(stream),
                    Err(_) => { ipv6_done = true; }
                }
            }
            result = &mut ipv4_fut, if !ipv4_done => {
                match result {
                    Ok(stream) => return Ok(stream),
                    Err(_) => { ipv4_done = true; }
                }
            }
        }
        if ipv6_done && ipv4_done {
            break;
        }
    }

    // Both initial attempts failed, try remaining addresses sequentially
    // Interleave remaining IPv4 and IPv6 per RFC 8305
    let mut remaining = Vec::new();
    let ipv4_remaining = ipv4.iter().skip(1);
    let ipv6_remaining = ipv6.iter().skip(1);

    let mut ipv4_iter = ipv4_remaining.peekable();
    let mut ipv6_iter = ipv6_remaining.peekable();

    // Interleave: take one from ipv4, then one from ipv6, alternating
    while ipv4_iter.peek().is_some() || ipv6_iter.peek().is_some() {
        if let Some(addr) = ipv4_iter.next() {
            remaining.push(*addr);
        }
        if let Some(addr) = ipv6_iter.next() {
            remaining.push(*addr);
        }
    }

    try_addrs_sequential(&remaining).await
}

/// Try connecting to addresses sequentially, returning the first success.
async fn try_addrs_sequential(addrs: &[SocketAddr]) -> Result<TcpStream, Error> {
    let mut last_err = None;
    for addr in addrs {
        match TcpStream::connect(addr).await {
            Ok(stream) => return Ok(stream),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err
        .unwrap_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::AddrNotAvailable,
                "no addresses to connect to",
            )
        })
        .into())
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
        timeout,
        tokio_tungstenite::client_async_tls(url.as_str(), conn),
    ))
    .await
    .map_err(|_| Error::Timeout)??;
    Ok(WebSocket::Tokio(stream))
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
        timeout,
        tokio_tungstenite::client_async_tls(url.as_str(), conn),
    ))
    .await
    .map_err(|_| Error::Timeout)??;
    Ok(WebSocket::Tor(stream))
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

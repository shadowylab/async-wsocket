// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Native

#[cfg(feature = "socks")]
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

#[cfg(feature = "tor")]
use arti_client::DataStream;
use async_utility::time;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{Sink as SinkTrait, Stream as StreamTrait, StreamExt};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as WsError;
pub use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use url::{ParseError, Url};

#[cfg(feature = "socks")]
mod socks;
#[cfg(feature = "tor")]
mod tor;

#[cfg(feature = "socks")]
use self::socks::TcpSocks5Stream;
use crate::ConnectionMode;

type WsStream<T> = WebSocketStream<MaybeTlsStream<T>>;

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

pub enum WebSocket {
    Std(WsStream<TcpStream>),
    #[cfg(feature = "tor")]
    Tor(WsStream<DataStream>),
}

pub enum Sink {
    Std(SplitSink<WsStream<TcpStream>, Message>),
    #[cfg(feature = "tor")]
    Tor(SplitSink<WsStream<DataStream>, Message>),
}

impl SinkTrait<Message> for Sink {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.deref_mut() {
            Self::Std(s) => Pin::new(s).poll_ready(cx).map_err(Into::into),
            #[cfg(feature = "tor")]
            Self::Tor(s) => Pin::new(s).poll_ready(cx).map_err(Into::into),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match self.deref_mut() {
            Self::Std(s) => Pin::new(s).start_send(item).map_err(Into::into),
            #[cfg(feature = "tor")]
            Self::Tor(s) => Pin::new(s).start_send(item).map_err(Into::into),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.deref_mut() {
            Self::Std(s) => Pin::new(s).poll_flush(cx).map_err(Into::into),
            #[cfg(feature = "tor")]
            Self::Tor(s) => Pin::new(s).poll_flush(cx).map_err(Into::into),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.deref_mut() {
            Self::Std(s) => Pin::new(s).poll_close(cx).map_err(Into::into),
            #[cfg(feature = "tor")]
            Self::Tor(s) => Pin::new(s).poll_close(cx).map_err(Into::into),
        }
    }
}

pub enum Stream {
    Std(SplitStream<WsStream<TcpStream>>),
    #[cfg(feature = "tor")]
    Tor(SplitStream<WsStream<DataStream>>),
}

impl StreamTrait for Stream {
    type Item = Result<Message, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.deref_mut() {
            Self::Std(s) => Pin::new(s).poll_next(cx).map_err(Into::into),
            #[cfg(feature = "tor")]
            Self::Tor(s) => Pin::new(s).poll_next(cx).map_err(Into::into),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Std(s) => s.size_hint(),
            #[cfg(feature = "tor")]
            Self::Tor(s) => s.size_hint(),
        }
    }
}

pub async fn connect(
    url: &Url,
    mode: ConnectionMode,
    timeout: Duration,
) -> Result<(Sink, Stream), Error> {
    let stream: WebSocket = match mode {
        ConnectionMode::Direct => connect_direct(url, timeout).await?,
        #[cfg(feature = "socks")]
        ConnectionMode::Proxy(proxy) => connect_proxy(url, proxy, timeout).await?,
        #[cfg(feature = "tor")]
        ConnectionMode::Tor => connect_tor(url, timeout).await?,
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
    let (stream, _) = time::timeout(
        Some(timeout),
        tokio_tungstenite::connect_async(url.as_str()),
    )
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
    let host: &str = url.host_str().ok_or(Error::Url(ParseError::EmptyHost))?;
    let port: u16 = url
        .port_or_known_default()
        .ok_or(Error::Url(ParseError::InvalidPort))?;
    let addr: String = format!("{host}:{port}");

    let conn: TcpStream = TcpSocks5Stream::connect(proxy, addr).await?;
    let (stream, _) = time::timeout(
        Some(timeout),
        tokio_tungstenite::client_async_tls(url.as_str(), conn),
    )
    .await
    .ok_or(Error::Timeout)??;
    Ok(WebSocket::Std(stream))
}

#[cfg(feature = "tor")]
async fn connect_tor(url: &Url, timeout: Duration) -> Result<WebSocket, Error> {
    let host: &str = url.host_str().ok_or(Error::Url(ParseError::EmptyHost))?;
    let port: u16 = url
        .port_or_known_default()
        .ok_or(Error::Url(ParseError::InvalidPort))?;

    let conn: DataStream = tor::connect(host, port).await?;
    let (stream, _) = time::timeout(
        Some(timeout),
        tokio_tungstenite::client_async_tls(url.as_str(), conn),
    )
    .await
    .ok_or(Error::Timeout)??;
    Ok(WebSocket::Tor(stream))
}

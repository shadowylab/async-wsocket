// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::DerefMut;
use std::pin::Pin;
use std::task::{Context, Poll};

#[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
use arti_client::DataStream;
use futures_util::{Sink, Stream};
#[cfg(not(target_arch = "wasm32"))]
use tokio::net::TcpStream;
#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

#[cfg(target_arch = "wasm32")]
use crate::wasm::WsStream;
use crate::{Error, Message};

#[cfg(not(target_arch = "wasm32"))]
type WsStream<T> = WebSocketStream<MaybeTlsStream<T>>;

pub enum WebSocket {
    #[cfg(not(target_arch = "wasm32"))]
    Tokio(WsStream<TcpStream>),
    #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
    Tor(WsStream<DataStream>),
    #[cfg(target_arch = "wasm32")]
    Wasm(WsStream),
}

impl Sink<Message> for WebSocket {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.deref_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(s) => Pin::new(s).poll_ready(cx).map_err(Into::into),
            #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
            Self::Tor(s) => Pin::new(s).poll_ready(cx).map_err(Into::into),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(s) => Pin::new(s).poll_ready(cx).map_err(Into::into),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match self.deref_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(s) => Pin::new(s).start_send(item.into()).map_err(Into::into),
            #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
            Self::Tor(s) => Pin::new(s).start_send(item.into()).map_err(Into::into),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(s) => Pin::new(s).start_send(item).map_err(Into::into),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.deref_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(s) => Pin::new(s).poll_flush(cx).map_err(Into::into),
            #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
            Self::Tor(s) => Pin::new(s).poll_flush(cx).map_err(Into::into),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(s) => Pin::new(s).poll_flush(cx).map_err(Into::into),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.deref_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(s) => Pin::new(s).poll_close(cx).map_err(Into::into),
            #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
            Self::Tor(s) => Pin::new(s).poll_close(cx).map_err(Into::into),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(s) => Pin::new(s).poll_close(cx).map_err(Into::into),
        }
    }
}

impl Stream for WebSocket {
    type Item = Result<Message, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.deref_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(s) => Pin::new(s)
                .poll_next(cx)
                .map(|i| i.map(|res| res.map(|msg| msg.into())))
                .map_err(Into::into),
            #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
            Self::Tor(s) => Pin::new(s)
                .poll_next(cx)
                .map(|i| i.map(|res| res.map(|msg| msg.into())))
                .map_err(Into::into),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(s) => Pin::new(s).poll_next(cx).map_err(Into::into),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Tokio(s) => s.size_hint(),
            #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
            Self::Tor(s) => s.size_hint(),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(s) => s.size_hint(),
        }
    }
}

// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::ops::DerefMut;
use std::pin::Pin;
use std::task::{Context, Poll};

#[cfg(feature = "tor")]
use arti_client::DataStream;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{Sink as SinkTrait, Stream as StreamTrait};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use super::error::Error;

type WsStream<T> = WebSocketStream<MaybeTlsStream<T>>;

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

// Copyright (c) 2019-2022 Naja Melan
// Copyright (c) 2023-2024 Yuki Kishimoto
// Distributed under the MIT software license

use std::io::{self, ErrorKind};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{Sink, Stream};

use crate::wasm::{WsError, WsStream};

/// A wrapper around WsStream that converts errors into io::Error so that it can be
/// used for io (like `AsyncRead`/`AsyncWrite`).
///
/// You shouldn't need to use this manually. It is passed to [`IoStream`] when calling
/// [`WsStream::into_io`].
#[derive(Debug)]
pub struct WsStreamIo {
    inner: WsStream,
}

impl Stream for WsStreamIo {
    type Item = Result<Vec<u8>, io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx).map(|opt| {
            opt.map(|msg| {
                msg.map(|m| m.into_data())
                    .map_err(|e| io::Error::new(ErrorKind::Other, e))
            })
        })
    }
}

impl Sink<Vec<u8>> for WsStreamIo {
    type Error = io::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner)
            .poll_ready(cx)
            .map(convert_res_tuple)
    }

    fn start_send(mut self: Pin<&mut Self>, item: Vec<u8>) -> Result<(), Self::Error> {
        Pin::new(&mut self.inner)
            .start_send(item.into())
            .map_err(convert_err)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner)
            .poll_flush(cx)
            .map(convert_res_tuple)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.inner)
            .poll_close(cx)
            .map(convert_res_tuple)
    }
}

#[inline]
fn convert_res_tuple(res: Result<(), WsError>) -> Result<(), io::Error> {
    res.map_err(convert_err)
}

fn convert_err(err: WsError) -> io::Error {
    match err {
        WsError::ConnectionNotOpen => io::Error::from(ErrorKind::NotConnected),
        // This shouldn't happen, so panic for early detection.
        _ => io::Error::from(ErrorKind::Other),
    }
}

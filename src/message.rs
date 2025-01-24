// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

#[cfg(not(target_arch = "wasm32"))]
use std::borrow::Cow;

#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::tungstenite::protocol::CloseFrame as TungsteniteCloseFrame;
#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::tungstenite::protocol::Message as TungsteniteMessage;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CloseFrame {
    /// The reason as a code.
    pub code: u16,
    /// The reason as text string.
    pub reason: String,
}

/// An enum representing the various forms of a WebSocket message.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Message {
    /// A text WebSocket message
    Text(String),
    /// A binary WebSocket message
    Binary(Vec<u8>),
    /// A ping message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    #[cfg(not(target_arch = "wasm32"))]
    Ping(Vec<u8>),
    /// A pong message with the specified payload
    ///
    /// The payload here must have a length less than 125 bytes
    #[cfg(not(target_arch = "wasm32"))]
    Pong(Vec<u8>),
    /// A close message with the optional close frame.
    #[cfg(not(target_arch = "wasm32"))]
    Close(Option<CloseFrame>),
}

// impl Message {
//     /// Get the length of the WebSocket message.
//     #[inline]
//     pub fn len(&self) -> usize {
//         match self {
//             Self::Text(string) => string.len(),
//             Self::Binary(data) => data.len(),
//             Self::Ping(data) => data.len(),
//             Self::Pong(data) => data.len(),
//             Self::Close(frame) => frame.map(|f| f.reason.len()).unwrap_or(0),
//         }
//     }
//
//     /// Returns true if the WebSocket message has no content.
//     /// For example, if the other side of the connection sent an empty string.
//     #[inline]
//     pub fn is_empty(&self) -> bool {
//         self.len() == 0
//     }
//
//     /// Consume the message and return it as binary data.
//     pub fn into_data(self) -> Vec<u8> {
//         match self {
//             Self::Text(string) => string.into_bytes(),
//             Self::Binary(data) => data,
//         }
//     }
// }

#[cfg(not(target_arch = "wasm32"))]
impl From<CloseFrame> for TungsteniteCloseFrame<'_> {
    fn from(frame: CloseFrame) -> Self {
        Self {
            code: CloseCode::from(frame.code),
            reason: Cow::Owned(frame.reason),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Message> for TungsteniteMessage {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Text(text) => Self::Text(text),
            Message::Binary(data) => Self::Binary(data),
            Message::Ping(data) => Self::Ping(data),
            Message::Pong(data) => Self::Pong(data),
            Message::Close(frame) => Self::Close(frame.map(|f| f.into())),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<TungsteniteCloseFrame<'_>> for CloseFrame {
    fn from(frame: TungsteniteCloseFrame<'_>) -> Self {
        Self {
            code: frame.code.into(),
            reason: frame.reason.into_owned(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<TungsteniteMessage> for Message {
    fn from(msg: TungsteniteMessage) -> Self {
        match msg {
            TungsteniteMessage::Text(text) => Self::Text(text),
            TungsteniteMessage::Binary(data) => Self::Binary(data),
            TungsteniteMessage::Ping(data) => Self::Ping(data),
            TungsteniteMessage::Pong(data) => Self::Pong(data),
            TungsteniteMessage::Close(frame) => Self::Close(frame.map(|f| f.into())),
            // SAFETY: from tungstenite docs: "you're not going to get this value while reading the message".
            // SAFETY: this conversion is used only in Stream trait, so when reading the messages.
            TungsteniteMessage::Frame(..) => unreachable!(),
        }
    }
}

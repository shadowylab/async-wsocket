// Copyright (c) 2019-2022 Naja Melan
// Copyright (c) 2023-2024 Yuki Kishimoto
// Distributed under the MIT software license

use core::{fmt, str};

use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::JsCast;
use web_sys::{Blob, MessageEvent};

use crate::wasm::Error;

/// Represents a WebSocket Message, after converting from JavaScript type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WsMessage {
    /// The data of the message is a string.
    Text(String),
    /// The message contains binary data.
    Binary(Vec<u8>),
}

impl WsMessage {
    /// Get the length of the WebSocket message.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Text(string) => string.len(),
            Self::Binary(data) => data.len(),
        }
    }

    /// Returns true if the WebSocket message has no content.
    /// For example, if the other side of the connection sent an empty string.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Consume the message and return it as binary data.
    pub fn into_data(self) -> Vec<u8> {
        match self {
            Self::Text(string) => string.into_bytes(),
            Self::Binary(data) => data,
        }
    }

    /// Attempt to get a &str from the WebSocket message,
    /// this will try to convert binary data to utf8.
    pub fn to_text(&self) -> Result<&str, Error> {
        match self {
            Self::Text(string) => Ok(string),
            Self::Binary(data) => Ok(str::from_utf8(data)?),
        }
    }
}

/// This will convert the JavaScript event into a WsMessage. Note that this
/// will only work if the connection is set to use the binary type ArrayBuffer.
/// On binary type Blob, this will panic.
impl TryFrom<MessageEvent> for WsMessage {
    type Error = Error;

    fn try_from(evt: MessageEvent) -> Result<Self, Self::Error> {
        match evt.data() {
            d if d.is_instance_of::<ArrayBuffer>() => Ok(WsMessage::Binary(
                Uint8Array::new(d.unchecked_ref()).to_vec(),
            )),

            // We don't allow invalid encodings. In principle if needed,
            // we could add a variant to WsMessage with a CString or an OsString
            // to allow the user to access this data. However until there is a usecase,
            // I'm not inclined, amongst other things because the conversion from Js isn't very
            // clear and it would require a bunch of testing for something that's a rather bad
            // idea to begin with. If you need data that is not a valid string, use a binary
            // message.
            d if d.is_string() => match d.as_string() {
                Some(text) => Ok(WsMessage::Text(text)),
                None => Err(Error::InvalidEncoding),
            },

            // We have set the binary mode to array buffer (WsMeta::connect), so normally this shouldn't happen.
            // That is as long as this is used within the context of the WsMeta constructor.
            d if d.is_instance_of::<Blob>() => Err(Error::CantDecodeBlob),

            // should never happen.
            _ => Err(Error::UnknownDataType),
        }
    }
}

impl From<Vec<u8>> for WsMessage {
    fn from(vec: Vec<u8>) -> Self {
        WsMessage::Binary(vec)
    }
}

impl From<String> for WsMessage {
    fn from(s: String) -> Self {
        WsMessage::Text(s)
    }
}

impl AsRef<[u8]> for WsMessage {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Text(string) => string.as_ref(),
            Self::Binary(data) => data.as_ref(),
        }
    }
}

impl fmt::Display for WsMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Ok(string) = self.to_text() {
            write!(f, "{string}")
        } else {
            write!(f, "Binary Data<length={}>", self.len())
        }
    }
}

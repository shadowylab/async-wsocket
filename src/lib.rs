// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Async WebSocket

#![forbid(unsafe_code)]

pub use futures_util;
pub use url_fork as url;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm_ws::WsMessage;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::Message as WsMessage;

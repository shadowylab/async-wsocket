// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Prelude

#![allow(unknown_lints)]
#![allow(ambiguous_glob_reexports)]
#![doc(hidden)]

// Internal modules
#[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
pub use crate::native::tor::*;
#[cfg(not(target_arch = "wasm32"))]
pub use crate::native::{self, *};
pub use crate::*;

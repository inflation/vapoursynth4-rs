/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(warnings)]

mod api;
mod constants;
mod core;
pub mod error;
mod frame;
mod function;
pub mod map;
mod node;
pub mod plugin;
mod utils;

pub use crate::core::*;
pub use api::*;
pub use constants::*;
pub use frame::*;
pub use function::*;
pub use map::*;
pub use node::*;
pub use utils::*;

use vapoursynth4_sys as ffi;

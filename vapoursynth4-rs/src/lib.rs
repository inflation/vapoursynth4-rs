/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod api;
pub mod core;
pub mod frame;
pub mod function;
pub mod map;
pub mod node;
pub mod plugin;
pub mod utils;

pub use api::*;

use vapoursynth4_sys as ffi;

mod _private {
    pub trait Sealed {}
}

pub type ColorFamily = ffi::VSColorFamily;
pub type SampleType = ffi::VSSampleType;

pub type VideoInfo = ffi::VSVideoInfo;
pub type AudioInfo = ffi::VSAudioInfo;

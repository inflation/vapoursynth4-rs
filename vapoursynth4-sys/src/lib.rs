/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

//! Raw bindings to [VapourSynth](https://github.com/vapoursynth/vapoursynth).

#![warn(clippy::pedantic)]
#![allow(clippy::wildcard_imports)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod constants;
pub mod helper;
mod vs;
mod vsscript;

pub use crate::constants::*;
pub use crate::vs::*;

#[cfg(any(feature = "vscript", feature = "vsscript-41"))]
pub use crate::vsscript::*;

macro_rules! opaque_struct {
    ($($(#[$outer:meta])*$name:ident),+) => {
        $(
            $(#[$outer])*
            #[repr(C)]
            pub struct $name {
                _data: [u8; 0],
                _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
            }
        )*
    };
}
pub(crate) use opaque_struct;

/// Used to create version numbers.
/// The first argument is the major version and second is the minor.
#[must_use]
pub const fn vs_make_version(major: u16, minor: u16) -> i32 {
    ((major as i32) << 16) | minor as i32
}

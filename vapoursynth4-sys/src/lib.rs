/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

//! Raw bindings to [VapourSynth](https://github.com/vapoursynth/vapoursynth).

#![warn(clippy::pedantic)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(clippy::wildcard_imports)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::cast_possible_wrap)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

mod constants;
pub mod helper;
mod vs;
mod vsscript;

pub use crate::constants::*;
pub use crate::vs::*;
pub use crate::vsscript::*;

#[macro_export]
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

#[must_use]
/// Used to create version numbers.
/// The first argument is the major version and second is the minor.
pub const fn VS_MAKE_VERSION(major: u16, minor: u16) -> i32 {
    ((major as i32) << 16) | minor as i32
}

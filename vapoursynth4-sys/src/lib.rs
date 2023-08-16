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
    ($($name:ident),+) => {
        $(
            #[repr(C)]
            pub struct $name {
                _data: [u8; 0],
                _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
            }
        )*
    };
}

#[must_use]
pub const fn VS_MAKE_VERSION(major: i32, minor: i32) -> i32 {
    (major << 16) | minor
}

pub const VAPOURSYNTH_API_VERSION: i32 = VS_MAKE_VERSION(4, 0);

pub const VSSCRIPT_API_VERSION: i32 = if cfg!(feature = "vsscript-api-41") {
    VS_MAKE_VERSION(4, 1)
} else {
    VS_MAKE_VERSION(4, 0)
};

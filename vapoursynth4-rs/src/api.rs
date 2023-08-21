/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{ops::Deref, ptr::NonNull};

use once_cell::sync::OnceCell;
use vapoursynth4_sys as ffi;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ApiRef {
    handle: NonNull<ffi::VSAPI>,
}

impl ApiRef {
    #[must_use]
    pub fn new() -> Option<Self> {
        Self::new_with_version(ffi::VAPOURSYNTH_API_VERSION)
    }

    #[must_use]
    pub fn new_with(major: u16, minor: u16) -> Option<Self> {
        Self::new_with_version(ffi::VS_MAKE_VERSION(major, minor))
    }

    #[must_use]
    pub fn new_with_version(version: i32) -> Option<Self> {
        let handle = NonNull::new(unsafe { ffi::getVapourSynthAPI(version) }.cast_mut())?;
        Some(Self { handle })
    }

    #[must_use]
    pub fn get_version(&self) -> i32 {
        unsafe { (self.getAPIVersion)() }
    }
}

impl Deref for ApiRef {
    type Target = ffi::VSAPI;

    fn deref(&self) -> &Self::Target {
        unsafe { self.handle.as_ref() }
    }
}

impl Default for ApiRef {
    /// # Panics
    ///
    /// Panic if [`getVapourSynthAPI()`](ffi::getVapourSynthAPI) returns `NULL`
    fn default() -> Self {
        Self::new().unwrap()
    }
}

unsafe impl Send for ApiRef {}
unsafe impl Sync for ApiRef {}

pub(crate) static API: OnceCell<ApiRef> = OnceCell::new();

pub fn api() -> &'static ApiRef {
    API.get_or_init(ApiRef::default)
}

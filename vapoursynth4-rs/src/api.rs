/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{ops::Deref, ptr::NonNull};

use once_cell::sync::OnceCell;

use crate::ffi;

pub(crate) static API: OnceCell<ApiRef> = OnceCell::new();

pub(crate) fn api() -> &'static ApiRef {
    API.get_or_init(ApiRef::default)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct ApiRef {
    handle: NonNull<ffi::VSAPI>,
}

impl ApiRef {
    /// # Safety
    #[must_use]
    pub unsafe fn from_raw(ptr: *const ffi::VSAPI) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
        }
    }

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

    pub fn set(self) -> Option<()> {
        API.set(self).ok()
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
        Self::new().expect(
            "Failed to get API with the version specified by features. \
             Check if the version is supported by the linked VapourSynth library, \
             or lowering the requirement in `Cargo.toml`.",
        )
    }
}

unsafe impl Send for ApiRef {}
unsafe impl Sync for ApiRef {}

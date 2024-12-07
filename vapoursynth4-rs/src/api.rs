/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use core::panic;
use std::{
    ops::Deref,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};

#[cfg(feature = "link-library")]
use vapoursynth4_sys::vs_make_version;

use crate::ffi;

#[cfg(feature = "link-library")]
use self::error::ApiNotFound;

static API: Api = Api {
    handle: AtomicPtr::new(std::ptr::null_mut()),
};

pub(crate) fn set_api(ptr: *const ffi::VSAPI) {
    API.set(ptr);
}

pub(crate) fn api() -> &'static Api {
    if API.handle.load(Ordering::Acquire).is_null() {
        panic!("API is not set");
    } else {
        &API
    }
}

#[derive(Debug)]
pub struct Api {
    pub(crate) handle: AtomicPtr<ffi::VSAPI>,
}

impl Api {
    /// Creates a new `Api` instance with the specified major and minor version.
    ///
    /// # Errors
    ///
    /// Returns `ApiNotFound` if the requested API version is not supported by the linked `VapourSynth` library.
    #[cfg(feature = "link-library")]
    pub fn new(major: u16, minor: u16) -> Result<Self, ApiNotFound> {
        let ptr = unsafe { ffi::getVapourSynthAPI(vs_make_version(major, minor)) };
        if ptr.is_null() {
            Err(ApiNotFound { major, minor })
        } else {
            Ok(Self {
                handle: AtomicPtr::new(ptr.cast_mut()),
            })
        }
    }


    pub(crate) fn set(&self, ptr: *const ffi::VSAPI) {
        assert!(
            self.handle
                .compare_exchange(
                    null_mut(),
                    ptr.cast_mut(),
                    Ordering::AcqRel,
                    Ordering::Relaxed
                )
                .is_ok(),
            "API is already set"
        );
    }

    #[cfg(test)]
    #[cfg(feature = "link-library")]
    pub(crate) fn set_default() {
        let api = Self::default();
        unsafe { *(&raw const API).cast_mut() = api };
    }
}

impl Deref for Api {
    type Target = ffi::VSAPI;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.handle.load(Ordering::Acquire) }
    }
}

#[cfg(feature = "link-library")]
impl Default for Api {
    /// Creates a new `Api` instance with the default version.
    ///
    /// # Panics
    ///
    /// Internal error indicates that something went wrong with the linked `VapourSynth` library.
    #[must_use]
    fn default() -> Self {
        Self::new(ffi::VAPOURSYNTH_API_MAJOR, ffi::VAPOURSYNTH_API_MINOR).unwrap()
    }
}

pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    #[error(
        "Request API with version {major}.{minor} failed. \
        Please check if the version is supported by the linked VapourSynth library."
    )]
    pub struct ApiNotFound {
        pub major: u16,
        pub minor: u16,
    }
}

/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::sync::atomic::{AtomicPtr, Ordering};

use crate::ffi;

use self::error::ApiNotFound;

pub(crate) static mut API: Option<AtomicPtr<ffi::VSAPI>> = None;

pub(crate) fn api() -> &'static ffi::VSAPI {
    unsafe {
        &*API
            .as_ref()
            .expect("Please set API first with `set_api` or `set_api_default`.")
            .load(Ordering::Acquire)
    }
}

/// # Errors
///
/// Return [`ApiNotFound`] if the requested API is not found.
pub fn set_api_default() -> Result<(), ApiNotFound> {
    let ptr = unsafe { ffi::getVapourSynthAPI(ffi::VAPOURSYNTH_API_VERSION) };
    if ptr.is_null() {
        Err(error::ApiNotFound {
            major: ffi::VAPOURSYNTH_API_MAJOR,
            minor: ffi::VAPOURSYNTH_API_MINOR,
        })
    } else {
        unsafe {
            API.replace(AtomicPtr::new(ptr.cast_mut()));
        }
        Ok(())
    }
}

/// # Errors
///
/// Return [`ApiNotFound`] if the requested API is not found.
pub fn set_api(major: u16, minor: u16) -> Result<(), ApiNotFound> {
    let version = ffi::vs_make_version(major, minor);
    let ptr = unsafe { ffi::getVapourSynthAPI(version) };
    if ptr.is_null() {
        Err(error::ApiNotFound { major, minor })
    } else {
        unsafe {
            API.replace(AtomicPtr::new(ptr.cast_mut()));
        }
        Ok(())
    }
}

pub(crate) unsafe fn set_api_from_raw(ptr: *const ffi::VSAPI) {
    API.replace(AtomicPtr::new(ptr.cast_mut()));
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

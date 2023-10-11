/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::sync::atomic::{AtomicPtr, Ordering};

use crate::ffi;

use self::error::ApiNotFound;

pub(crate) static mut API: Api = Api::null();

pub(crate) fn api() -> &'static ffi::VSAPI {
    unsafe { &*API.handle.load(Ordering::Acquire) }
}

#[repr(transparent)]
pub(crate) struct Api {
    handle: AtomicPtr<ffi::VSAPI>,
}

impl Api {
    const fn null() -> Self {
        Self {
            handle: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    pub(crate) fn default() -> Result<*const ffi::VSAPI, ApiNotFound> {
        let ptr = unsafe { ffi::getVapourSynthAPI(ffi::VAPOURSYNTH_API_VERSION) };
        if ptr.is_null() {
            Err(ApiNotFound {
                major: ffi::VAPOURSYNTH_API_MAJOR,
                minor: ffi::VAPOURSYNTH_API_MINOR,
            })
        } else {
            Ok(ptr)
        }
    }

    pub(crate) fn set(&mut self, ptr: *const ffi::VSAPI) {
        self.handle.store(ptr.cast_mut(), Ordering::Release);
    }

    #[cfg(test)]
    pub(crate) fn set_default(&mut self) -> Result<(), ApiNotFound> {
        Self::default().map(|api| self.set(api))
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

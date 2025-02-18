/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::ops::Deref;

#[cfg(feature = "link-library")]
use vapoursynth4_sys::vs_make_version;

use crate::ffi;

#[cfg(feature = "link-library")]
use self::error::ApiNotFound;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Api(pub(crate) *const ffi::VSAPI);

impl Api {
    /// Creates a new `Api` instance with the specified major and minor version.
    ///
    /// # Errors
    ///
    /// Returns `ApiNotFound` if the requested API version is not supported by the linked `VapourSynth` library.
    #[cfg(feature = "link-library")]
    pub fn new(major: u16, minor: u16) -> Result<Self, ApiNotFound> {
        let ptr = unsafe { ffi::getVapourSynthAPI(vs_make_version(major, minor)) };
        (!ptr.is_null())
            .then_some(Self(ptr))
            .ok_or(ApiNotFound { major, minor })
    }
}

impl Deref for Api {
    type Target = ffi::VSAPI;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
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
        Self::new(ffi::VAPOURSYNTH_API_MAJOR, ffi::VAPOURSYNTH_API_MINOR)
            .expect("Failed to get VSAPI. Please check if the dynamic library is linked correctly")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct VssApi(*const ffi::VSSCRIPTAPI);

impl VssApi {
    /// Creates a new `VssApi` instance with the specified major and minor version.
    ///
    /// # Errors
    ///
    /// Returns `ApiNotFound` if the requested API version is not supported by the linked `VapourSynth` library.
    #[cfg(feature = "link-library")]
    pub fn new(major: u16, minor: u16) -> Result<Self, ApiNotFound> {
        let ptr = unsafe { ffi::getVSScriptAPI(vs_make_version(major, minor)) };
        (!ptr.is_null())
            .then_some(Self(ptr))
            .ok_or(ApiNotFound { major, minor })
    }

    #[allow(unused)]
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::VSSCRIPTAPI) -> Self {
        Self(ptr.cast_mut())
    }
}

impl Deref for VssApi {
    type Target = ffi::VSSCRIPTAPI;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

#[cfg(feature = "link-library")]
impl Default for VssApi {
    /// Creates a new `Api` instance with the default version.
    ///
    /// # Panics
    ///
    /// Internal error indicates that something went wrong with the linked `VapourSynth` library.
    #[must_use]
    fn default() -> Self {
        Self::new(ffi::VSSCRIPT_API_MAJOR, ffi::VSSCRIPT_API_MINOR).expect("Failed to get VSSCRIPTAPI. 
        Please check if the dynamic library is linked correctly and the proper Python environment is selected")
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

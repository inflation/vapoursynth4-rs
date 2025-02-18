/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{
    ffi::{c_void, CStr},
    ptr::null_mut,
};

use crate::{
    core::CoreRef,
    ffi,
    frame::{Frame, FrameContext, FrameType},
    map::MapRef,
    node::FilterMode,
};

pub trait Filter
where
    Self: Sized,
{
    /// Filter error that can turned into a [`&CStr`](std::ffi::CStr)
    type Error: AsRef<CStr>;
    type FrameType: FrameType;
    type FilterData: 'static + Send + Sync;

    const NAME: &'static FilterName;
    const ARGS: &'static CStr;
    const RETURN_TYPE: &'static CStr;
    const FILTER_MODE: FilterMode = FilterMode::Parallel;

    /// # Errors
    ///
    /// Return [`Self::Error`] if anything happens during the filter creation.
    /// The error message will be passed to `VapourSynth`.
    fn create(
        input: MapRef,
        output: MapRef,
        data: Option<&Self::FilterData>,
        core: CoreRef,
    ) -> Result<(), Self::Error>;

    /// # Errors
    ///
    /// Return [`Self::Error`] if anything happens during the filter creation.
    /// The error message will be passed to `VapourSynth`.
    fn get_frame(
        &self,
        n: i32,
        activation_reason: ffi::VSActivationReason,
        frame_data: &mut [*mut c_void; 4],
        frame_ctx: FrameContext,
        core: CoreRef,
    ) -> Result<Option<Frame<Self::FrameType>>, Self::Error>;

    /// Free the filter
    fn free(self, core: CoreRef) {
        let _ = core;
    }
}

pub struct FilterRegister<F: Filter> {
    data: Option<F::FilterData>,
}

impl<F: Filter> FilterRegister<F> {
    pub fn new(data: Option<F::FilterData>) -> Self {
        Self { data }
    }

    /// # Safety
    #[doc(hidden)]
    pub unsafe fn register(
        self,
        plugin: *mut ffi::VSPlugin,
        vspapi: *const ffi::VSPLUGINAPI,
    ) -> Result<(), &'static str> {
        use super::internal::FilterExtern;

        let err = ((*vspapi).registerFunction)(
            F::NAME.0.as_ptr(),
            F::ARGS.as_ptr(),
            F::RETURN_TYPE.as_ptr(),
            F::filter_create,
            self.data
                .map_or(null_mut(), |d| Box::into_raw(Box::new(d)).cast()),
            plugin,
        );
        if err == 0 {
            return Err("Register function failed");
        }

        Ok(())
    }
}

pub type ActivationReason = ffi::VSActivationReason;

#[repr(transparent)]
pub struct FilterName(CStr);

impl FilterName {
    #[must_use]
    pub const fn from_cstr(name: &CStr) -> &Self {
        let len = name.count_bytes();
        let k = name.to_bytes();
        assert!(
            k[0].is_ascii_alphabetic(),
            "Filter name must start with an alphabetic character"
        );
        let mut i = 1;

        while i < len {
            assert!(
                k[i].is_ascii_alphanumeric() || k[i] == b'_',
                "Filter name must only contain alphanumeric characters or underscores"
            );
            i += 1;
        }
        unsafe { &*(std::ptr::from_ref(name) as *const Self) }
    }
}

#[macro_export]
macro_rules! filter_name {
    ($s:literal) => {
        const {
            FilterName::from_cstr(unsafe {
                CStr::from_bytes_with_nul_unchecked(concat!($s, "\0").as_bytes())
            })
        }
    };
}

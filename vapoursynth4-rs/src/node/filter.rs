use std::{
    ffi::{CStr, c_void},
    ptr::{from_ref, null_mut},
};

use crate::{
    core::CoreRef,
    ffi,
    frame::{Frame, FrameContext},
    map::{MapMut, MapRef},
    node::FilterMode,
};

/// A filter instance that VapourSynth can move between worker threads and call concurrently.
pub trait Filter: Sized + Send + Sync + std::panic::RefUnwindSafe + 'static {
    /// Bitmask of [`ffi::VSFilterFlags`] used when creating video nodes.
    /// GPU output must agree with [`Self::RETURN_TYPE`]; defaults to CPU output.
    #[cfg(feature = "vs-43")]
    const FLAGS: i32 = 0;

    const FILTER_MODE: FilterMode = FilterMode::Parallel;
    /// Filter error that can turned into a [`&CStr`](std::ffi::CStr)
    type Error: AsRef<CStr>;
    type FrameType: Frame;
    /// Shared registration data available to each call to [`Self::create`].
    type FilterData: Sync + 'static;

    const NAME: &'static CStr;
    const ARGS: &'static CStr;
    const RETURN_TYPE: &'static CStr;

    /// # Errors
    ///
    /// Return [`Self::Error`] if anything happens during the filter creation.
    /// The error message will be passed to `VapourSynth`.
    fn create(
        input: MapRef,
        output: MapMut,
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
        frame_data: *mut *mut c_void,
        frame_ctx: FrameContext,
        core: CoreRef,
    ) -> Result<Option<Self::FrameType>, Self::Error>;
    /// Free the filter
    fn free(self, core: CoreRef) {
        let _ = core;
    }
}

pub struct FilterRegister<F: Filter> {
    data: Option<&'static F::FilterData>,
}

impl<F: Filter> FilterRegister<F> {
    pub fn new(data: Option<&'static F::FilterData>) -> Self {
        Self { data }
    }

    /// # Safety
    ///
    /// Both pointers must be valid for a VapourSynth plugin registration call.
    pub unsafe fn register(self, plugin: *mut ffi::VSPlugin, vspapi: *const ffi::VSPLUGINAPI) {
        use super::internal::FilterExtern;

        unsafe {
            ((*vspapi).registerFunction)(
                F::NAME.as_ptr(),
                F::ARGS.as_ptr(),
                F::RETURN_TYPE.as_ptr(),
                F::filter_create,
                self.data
                    .map_or(null_mut(), |data| from_ref(data).cast_mut().cast()),
                plugin,
            )
        };
    }
}

pub type ActivationReason = ffi::VSActivationReason;

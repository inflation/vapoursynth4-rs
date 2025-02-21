use std::{
    ffi::{CStr, c_void},
    ptr::null_mut,
};

use crate::{
    core::CoreRef,
    ffi,
    frame::{Frame, FrameContext},
    map::MapRef,
    node::FilterMode,
};

pub trait Filter
where
    Self: Sized + std::panic::RefUnwindSafe,
{
    const FILTER_MODE: FilterMode = FilterMode::Parallel;
    /// Filter error that can turned into a [`&CStr`](std::ffi::CStr)
    type Error: AsRef<CStr>;
    type FrameType: Frame;
    type FilterData; // TODO: Ensure Send + Sync when FILTER_MODE is Parallel

    const NAME: &'static CStr;
    const ARGS: &'static CStr;
    const RETURN_TYPE: &'static CStr;

    /// # Errors
    ///
    /// Return [`Self::Error`] if anything happens during the filter creation.
    /// The error message will be passed to `VapourSynth`.
    fn create(
        input: MapRef,
        output: MapRef,
        data: Option<Box<Self::FilterData>>,
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
    data: Option<F::FilterData>,
}

impl<F: Filter> FilterRegister<F> {
    pub fn new(data: Option<F::FilterData>) -> Self {
        Self { data }
    }

    /// # Safety
    pub unsafe fn register(self, plugin: *mut ffi::VSPlugin, vspapi: *const ffi::VSPLUGINAPI) {
        use super::internal::FilterExtern;

        unsafe {
            ((*vspapi).registerFunction)(
                F::NAME.as_ptr(),
                F::ARGS.as_ptr(),
                F::RETURN_TYPE.as_ptr(),
                F::filter_create,
                self.data
                    .map_or(null_mut(), |d| Box::into_raw(Box::new(d)).cast()),
                plugin,
            )
        };
    }
}

pub type ActivationReason = ffi::VSActivationReason;

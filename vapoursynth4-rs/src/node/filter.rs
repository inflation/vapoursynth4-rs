use std::{
    ffi::{c_int, c_void, CStr},
    mem::ManuallyDrop,
    panic::AssertUnwindSafe,
    ptr::{null, null_mut},
};

use crate::{
    core::CoreRef,
    ffi,
    frame::{Frame, FrameContext},
    map::{MapMut, MapRef},
    node::FilterMode,
    set_api_from_raw,
    utils::ToCString,
};

pub trait Filter
where
    Self: Sized + std::panic::RefUnwindSafe,
{
    const FILTER_MODE: FilterMode = FilterMode::Parallel;
    /// Filter error that can turned into a [`&CStr`](std::ffi::CStr)
    type Error: AsRef<CStr>;
    type FrameType: Frame;
    type FilterData;

    const NAME: &'static CStr;
    const ARGS: &'static CStr;
    const RETURN_TYPE: &'static CStr;

    /// # Errors
    ///
    /// Return [`Self::Error`] if anything happens during the filter creation.
    /// The error message will be passed to `VapourSynth`.
    fn create(
        input: MapRef<'_>,
        output: MapMut<'_>,
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
    fn free(self, _core: CoreRef) {}
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
    pub unsafe fn register(self, plugin: *mut ffi::VSPlugin, vspapi: *const ffi::VSPLUGINAPI) {
        use self::internal::FilterExtern;

        ((*vspapi).registerFunction)(
            F::NAME.as_ptr(),
            F::ARGS.as_ptr(),
            F::RETURN_TYPE.as_ptr(),
            F::filter_create,
            self.data
                .map_or(null_mut(), |d| Box::into_raw(Box::new(d)).cast()),
            plugin,
        );
    }
}

pub(crate) mod internal {
    use super::{
        c_int, c_void, ffi, null, set_api_from_raw, AssertUnwindSafe, CoreRef, Filter, Frame,
        FrameContext, ManuallyDrop, MapMut, MapRef, ToCString,
    };

    pub trait FilterExtern: Filter {
        unsafe extern "system" fn filter_create(
            in_: *const ffi::VSMap,
            out: *mut ffi::VSMap,
            user_data: *mut c_void,
            core: *mut ffi::VSCore,
            vsapi: *const ffi::VSAPI,
        ) {
            set_api_from_raw(vsapi);

            let input = MapRef::from_ptr(in_);
            let mut output = MapMut::from_ptr(out);
            let core = CoreRef::from_ptr(core);
            let data = if user_data.is_null() {
                None
            } else {
                Some(Box::from_raw(user_data.cast()))
            };

            match std::panic::catch_unwind(AssertUnwindSafe(|| {
                Self::create(input, output, data, core)
            })) {
                Ok(Err(e)) => {
                    output.set_error(e.as_ref());
                }
                Err(p) => {
                    let e = p.downcast::<&str>().unwrap_unchecked();
                    output.set_error(&e.into_cstring_lossy());
                }
                _ => {}
            }
        }

        unsafe extern "system" fn filter_get_frame(
            n: c_int,
            activation_reason: ffi::VSActivationReason,
            instance_data: *mut c_void,
            frame_data: *mut *mut c_void,
            frame_ctx: *mut ffi::VSFrameContext,
            core: *mut ffi::VSCore,
            vsapi: *const ffi::VSAPI,
        ) -> *const ffi::VSFrame {
            let filter = instance_data.cast::<Self>().as_mut().unwrap_unchecked();
            let mut ctx = AssertUnwindSafe(FrameContext::from_ptr(frame_ctx));
            let core = CoreRef::from_ptr(core);
            set_api_from_raw(vsapi);

            match std::panic::catch_unwind(|| {
                let ctx = *ctx;
                filter.get_frame(n, activation_reason, frame_data, ctx, core)
            }) {
                Ok(Ok(Some(frame))) => {
                    // Transfer the ownership to VapourSynth
                    let frame = ManuallyDrop::new(frame);
                    return frame.as_ptr();
                }
                Ok(Err(e)) => {
                    ctx.set_filter_error(e.as_ref());
                }
                Err(p) => {
                    let e = p.downcast::<&str>().unwrap_unchecked();
                    ctx.set_filter_error(&e.into_cstring_lossy());
                }
                _ => {}
            }

            null()
        }

        unsafe extern "system" fn filter_free(
            instance_data: *mut c_void,
            core: *mut ffi::VSCore,
            vsapi: *const ffi::VSAPI,
        ) {
            let filter = Box::from_raw(instance_data.cast::<Self>());
            let core = CoreRef::from_ptr(core);
            set_api_from_raw(vsapi);

            filter.free(core);
        }
    }

    impl<F> FilterExtern for F where F: Filter {}
}

pub type ActivationReason = ffi::VSActivationReason;

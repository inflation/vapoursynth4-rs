use std::{
    ffi::{c_int, c_void, CStr},
    mem::ManuallyDrop,
    panic::AssertUnwindSafe,
    ptr::null,
};

use crate::{ffi, ApiRef, CoreRef, FilterMode, Frame, FrameContext, MapMut, MapRef, ToCString};

pub trait Filter
where
    Self: Sized + std::panic::RefUnwindSafe,
{
    type Error: AsRef<CStr>;
    type FrameType: Frame;
    const FILTER_MODE: FilterMode = FilterMode::Parallel;

    fn create<T>(
        input: MapRef<'_>,
        output: MapMut<'_>,
        data: Option<&mut T>,
        core: CoreRef,
    ) -> Result<(), Self::Error>;
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

pub trait FilterExtern: Filter {
    unsafe extern "system" fn filter_create(
        in_: *const ffi::VSMap,
        out: *mut ffi::VSMap,
        user_data: *mut c_void,
        core: *mut ffi::VSCore,
        vsapi: *const ffi::VSAPI,
    ) {
        let api = ApiRef::from_raw(vsapi);
        api.set().expect("API already set");
        let input = MapRef::from_ptr(in_);
        let mut output = MapMut::from_ptr(out);
        let core = CoreRef::from_ptr(core);
        let data = user_data.as_mut();

        match std::panic::catch_unwind(AssertUnwindSafe(|| Self::create(input, output, data, core)))
        {
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
        let api = ApiRef::from_raw(vsapi);
        _ = api.set();

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
        let api = ApiRef::from_raw(vsapi);
        _ = api.set();

        filter.free(core);
    }
}

impl<F> FilterExtern for F where F: Filter {}

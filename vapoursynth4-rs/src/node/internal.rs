use std::{
    ffi::{c_int, c_void},
    mem::ManuallyDrop,
    panic::AssertUnwindSafe,
    ptr::null,
};

use crate::{
    api::API,
    core::CoreRef,
    frame::{Frame, FrameContext},
    map::MapRef,
    utils::ToCString,
};

use super::{ffi, Filter};

pub trait FilterExtern: Filter {
    unsafe extern "system" fn filter_create(
        in_: *const ffi::VSMap,
        out: *mut ffi::VSMap,
        user_data: *mut c_void,
        core: *mut ffi::VSCore,
        vsapi: *const ffi::VSAPI,
    ) {
        API.set(vsapi);

        let input = MapRef::from_ptr(in_);
        let output = MapRef::from_ptr_mut(out);
        let core = CoreRef::from_ptr(core);
        let data = if user_data.is_null() {
            None
        } else {
            Some(Box::from_raw(user_data.cast()))
        };

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
        API.set(vsapi);

        let frame = std::panic::catch_unwind(|| {
            let ctx = *ctx;
            filter.get_frame(n, activation_reason, frame_data, ctx, core)
        });
        match frame {
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
        API.set(vsapi);

        filter.free(core);
    }
}

impl<F> FilterExtern for F where F: Filter {}

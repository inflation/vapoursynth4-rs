/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{
    ffi::{c_int, c_void},
    mem::ManuallyDrop,
    panic::AssertUnwindSafe,
    ptr::null_mut,
};

use crate::{api::Api, core::CoreRef, frame::FrameContext, map::MapRef, utils::ToCString};

use super::{ffi, Filter};

pub trait FilterExtern: Filter {
    unsafe extern "system-unwind" fn filter_create(
        in_: *const ffi::VSMap,
        out: *mut ffi::VSMap,
        user_data: *mut c_void,
        core: *mut ffi::VSCore,
        vsapi: *const ffi::VSAPI,
    ) {
        let api = Api(vsapi);
        let output = MapRef::from_ptr(out, api);

        let input = MapRef::from_ptr(in_, api);
        let core = CoreRef::from_ptr(core, api);
        let data = if user_data.is_null() {
            None
        } else {
            Some(&*user_data.cast())
        };

        let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
            std::panic::set_hook({
                Box::new(move |p| {
                    let e = format!("panic: {p}");
                    output.set_error(&e.into_cstring_lossy());
                })
            });
            if let Err(e) = Self::create(input, output, data, core) {
                output.set_error(e.as_ref());
            }
        }));
    }

    unsafe extern "system-unwind" fn filter_get_frame(
        n: c_int,
        activation_reason: ffi::VSActivationReason,
        instance_data: *mut c_void,
        frame_data: *mut *mut c_void,
        frame_ctx: *mut ffi::VSFrameContext,
        core: *mut ffi::VSCore,
        vsapi: *const ffi::VSAPI,
    ) -> *const ffi::VSFrame {
        let api = Api(vsapi);
        let ctx = FrameContext::from_ptr(frame_ctx, api);

        let filter = instance_data.cast::<Self>().as_mut().unwrap_unchecked();
        let core = CoreRef::from_ptr(core, api);
        let frame_data = &mut *frame_data.cast();

        let res = std::panic::catch_unwind(AssertUnwindSafe(|| {
            std::panic::set_hook({
                Box::new(move |p| {
                    let e = format!("panic: {p}");
                    ctx.set_filter_error(&e.into_cstring_lossy());
                })
            });

            let frame = filter.get_frame(n, activation_reason, frame_data, ctx, core);
            match frame {
                Ok(Some(frame)) => {
                    // Transfer the ownership to VapourSynth
                    let frame = ManuallyDrop::new(frame);
                    return frame.as_ptr();
                }
                Err(e) => {
                    ctx.set_filter_error(e.as_ref());
                }
                _ => {}
            }

            null_mut()
        }));

        res.unwrap_or(null_mut())
    }

    unsafe extern "system-unwind" fn filter_free(
        instance_data: *mut c_void,
        core: *mut ffi::VSCore,
        vsapi: *const ffi::VSAPI,
    ) {
        let api = Api(vsapi);
        let filter = Box::from_raw(instance_data.cast::<Self>());
        let core = CoreRef::from_ptr(core, api);

        filter.free(core);
    }
}

impl<F> FilterExtern for F where F: Filter {}

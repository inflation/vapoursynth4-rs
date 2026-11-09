use std::{
    any::Any,
    borrow::Cow,
    ffi::{CStr, CString, c_int, c_void},
    mem::{ManuallyDrop, forget},
    panic::{AssertUnwindSafe, catch_unwind},
    ptr::null,
};

use crate::{
    api::Api,
    core::CoreRef,
    frame::{Frame, FrameContext},
    map::{MapMut, MapRef},
    utils::ToCString,
};

use super::{Filter, ffi};

const PANIC_MESSAGE: &CStr = c"Rust filter panicked";

fn panic_text(payload: &(dyn Any + Send)) -> Option<Cow<'_, str>> {
    let text = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .or_else(|| payload.downcast_ref::<Box<str>>().map(Box::as_ref))
        .or_else(|| payload.downcast_ref::<Cow<'static, str>>().map(Cow::as_ref));
    text.map(Cow::Borrowed).or_else(|| {
        payload
            .downcast_ref::<&CStr>()
            .copied()
            .or_else(|| payload.downcast_ref::<CString>().map(CString::as_c_str))
            .or_else(|| payload.downcast_ref::<Box<CStr>>().map(Box::as_ref))
            .map(CStr::to_string_lossy)
    })
}

#[cold]
fn report_panic(payload: Box<dyn Any + Send>, report: impl FnOnce(&CStr)) {
    let message = panic_text(payload.as_ref())
        .map(|text| format!("{}: {text}", PANIC_MESSAGE.to_string_lossy()).into_cstring_lossy());

    // A custom payload's destructor can panic. Dropping that second payload can panic again.
    if let Err(payload) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
        forget(payload);
    }
    report(message.as_deref().unwrap_or(PANIC_MESSAGE));
}

pub trait FilterExtern: Filter {
    unsafe extern "system-unwind" fn filter_create(
        in_: *const ffi::VSMap,
        out: *mut ffi::VSMap,
        user_data: *mut c_void,
        core: *mut ffi::VSCore,
        vsapi: *const ffi::VSAPI,
    ) {
        unsafe {
            let api = Api::from_ptr(vsapi);

            let input = MapRef::from_ptr(in_, api);
            let mut output = MapMut::from_ptr(out, api);
            let core = CoreRef::from_ptr(core, api);
            let data = user_data.cast::<Self::FilterData>().as_ref();

            if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
                if let Err(error) = Self::create(input, output.reborrow(), data, core) {
                    output.set_error(error.as_ref());
                }
            })) {
                report_panic(payload, |message| output.set_error(message));
            }
        }
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
        unsafe {
            let api = Api::from_ptr(vsapi);
            let filter = &*instance_data.cast::<Self>();
            let mut ctx = FrameContext::from_ptr(frame_ctx, api);
            let core = CoreRef::from_ptr(core, api);

            let frame = catch_unwind(AssertUnwindSafe(|| {
                match filter.get_frame(n, activation_reason, frame_data, ctx, core) {
                    Ok(frame) => frame,
                    Err(error) => {
                        ctx.set_filter_error(error.as_ref());
                        None
                    }
                }
            }));
            match frame {
                Ok(Some(frame)) => {
                    // Transfer the ownership to VapourSynth
                    let frame = ManuallyDrop::new(frame);
                    return frame.as_ptr();
                }
                Err(payload) => {
                    report_panic(payload, |message| ctx.set_filter_error(message));
                }
                Ok(None) => {}
            }
        }

        null()
    }

    unsafe extern "system-unwind" fn filter_free(
        instance_data: *mut c_void,
        core: *mut ffi::VSCore,
        vsapi: *const ffi::VSAPI,
    ) {
        unsafe {
            let api = Api::from_ptr(vsapi);
            let filter = Box::from_raw(instance_data.cast::<Self>());
            if let Err(payload) = catch_unwind(AssertUnwindSafe(|| {
                filter.free(CoreRef::from_ptr(core, api));
            })) {
                report_panic(payload, |message| {
                    (api.logMessage)(ffi::VSMessageType::Critical, message.as_ptr(), core);
                });
            }
        }
    }
}

impl<F> FilterExtern for F where F: Filter {}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "link-vs")]
    #[test]
    fn factory_can_reuse_registration_data() {
        use crate::{core::Core, frame::VideoFrame};

        struct TestFilter;

        impl Filter for TestFilter {
            type Error = &'static CStr;
            type FrameType = VideoFrame;
            type FilterData = &'static CStr;

            const NAME: &'static CStr = c"Test";
            const ARGS: &'static CStr = c"";
            const RETURN_TYPE: &'static CStr = c"";

            fn create(
                _: MapRef<'_>,
                _: MapMut<'_>,
                data: Option<&Self::FilterData>,
                _: CoreRef<'_>,
            ) -> Result<(), Self::Error> {
                Err(data.copied().unwrap_or(c"missing registration data"))
            }

            fn get_frame(
                &self,
                _: i32,
                _: ffi::VSActivationReason,
                _: *mut *mut c_void,
                _: FrameContext,
                _: CoreRef<'_>,
            ) -> Result<Option<Self::FrameType>, Self::Error> {
                Ok(None)
            }
        }

        static DATA: &CStr = c"registration data";
        let core = Core::builder().disable_auto_loading().build();
        let input = core.create_map();
        let user_data = std::ptr::from_ref(&DATA).cast_mut().cast();
        for _ in 0..2 {
            let output = core.create_map();
            unsafe {
                TestFilter::filter_create(
                    input.as_ptr(),
                    output.as_ptr(),
                    user_data,
                    core.as_ptr(),
                    &raw const *core.api(),
                );
            }
            assert_eq!(output.get_error(), Some(DATA));
        }
    }

    #[test]
    fn panic_text_is_appended_to_the_common_message() {
        let payloads: [Box<dyn Any + Send>; 7] = [
            Box::new("detail"),
            Box::new(String::from("de\0tail")),
            Box::new(String::from("detail").into_boxed_str()),
            Box::new(Cow::Borrowed("detail")),
            Box::new(c"detail"),
            Box::new(c"detail".to_owned()),
            Box::new(c"detail".to_owned().into_boxed_c_str()),
        ];
        for payload in payloads {
            report_panic(payload, |message| {
                assert_eq!(message, c"Rust filter panicked: detail");
            });
        }
        report_panic(Box::new(42), |message| assert_eq!(message, PANIC_MESSAGE));
    }

    #[cfg(panic = "unwind")]
    #[test]
    fn panic_payload_cleanup_cannot_unwind() {
        struct PanicsOnDrop;

        impl Drop for PanicsOnDrop {
            fn drop(&mut self) {
                std::panic::panic_any(());
            }
        }

        report_panic(Box::new(PanicsOnDrop), |message| {
            assert_eq!(message, PANIC_MESSAGE);
        });
    }
}

use std::{
    ffi::{c_void, CString},
    ptr::null_mut,
};

use const_str::cstr;
use ffi::helper::isConstantVideoFormat;
use vapoursynth4_rs::{
    key, ApiRef, CoreRef, Filter, Frame, FrameContext, MapMut, MapRef, NodeRef, ToCString, Value,
};
use vapoursynth4_sys as ffi;

struct DumbFilter {
    node: NodeRef,
    enabled: bool,
}

impl Filter for DumbFilter {
    type Error = CString;
    // const FILTER_MODE: vapoursynth4_rs::FilterMode = vapoursynth4_rs::FilterMode::Unordered;

    fn get_frame(
        &self,
        n: i32,
        activation_reason: ffi::VSActivationReason,
        _frame_data: *mut *mut c_void,
        mut ctx: FrameContext,
        core: CoreRef,
    ) -> Result<Option<Frame>, Self::Error> {
        use ffi::VSActivationReason as r;

        match activation_reason {
            r::arInitial => {
                ctx.request_frame_filter(n, &self.node);
            }
            r::arAllFramesReady => {
                let src = ctx.get_frame_filter(n, &self.node);
                if !self.enabled {
                    panic!("Not enabled");
                }

                let fi = unsafe { src.get_video_format() };
                let height = src.frame_height(0).unwrap();
                let width = src.frame_width(0).unwrap();

                let mut dst = core.new_video_frame(fi, width, height, Some(&src));

                for plane in 0..fi.numPlanes {
                    let mut src_p = src.plane(plane);
                    let src_stride = src.stride(plane).unwrap();
                    let mut dst_p = dst.plane_mut(plane);
                    let dst_stride = dst.stride(plane).unwrap();

                    let h = src.frame_height(plane).unwrap();
                    let w = src.frame_width(plane).unwrap();

                    for _ in 0..h {
                        for x in 0..w as usize {
                            unsafe { *dst_p.wrapping_add(x) = !*src_p.wrapping_add(x) };
                        }

                        src_p = src_p.wrapping_offset(src_stride);
                        dst_p = dst_p.wrapping_offset(dst_stride);
                    }
                }
                return Ok(Some(dst));
            }
            _ => {}
        }

        Ok(None)
    }
}

/// # Safety
pub unsafe extern "system" fn filter_create(
    in_: *const ffi::VSMap,
    out: *mut ffi::VSMap,
    _user_data: *mut c_void,
    core: *mut ffi::VSCore,
    vsapi: *const ffi::VSAPI,
) {
    let api = ApiRef::from_raw(vsapi);
    api.set().expect("API already set");
    let in_ = MapRef::from_ptr(in_);
    let mut out = MapMut::from_ptr(out);
    let mut core = CoreRef::from_ptr(core);

    if let Err(e) = std::panic::catch_unwind(move || {
        let node = match in_.get(key!("clip"), 0) {
            Ok(Value::VideoNode(node)) => node,
            _ => {
                panic!("Failed to get node");
            }
        };
        let vi = node.get_video_info();
        if !isConstantVideoFormat(&*vi)
            || (*vi).format.sampleType != ffi::VSSampleType::stInteger
            || (*vi).format.bitsPerSample != 8
        {
            panic!("Invert: only constant format 8bit integer input supported");
        }

        let mut filter = DumbFilter {
            node,
            enabled: in_
                .get_int(key!("enabled"), 0)
                .map(|v| v != 0)
                .unwrap_or(true),
        };

        let deps = [ffi::VSFilterDependency {
            source: filter.node.as_mut_ptr(),
            requestPattern: ffi::VSRequestPattern::rpStrictSpatial,
        }];

        if let Err(e) = core.create_video_filter(out, "Invert", &*vi, Box::new(filter), &deps) {
            out.set_error(&e.to_string().into_cstring_lossy());
        }
    }) {
        let msg = e.downcast::<&str>().unwrap_unchecked().into_cstring_lossy();
        out.set_error(&msg);
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "system" fn VapourSynthPluginInit2(
    plugin: *mut ffi::VSPlugin,
    vspapi: *const ffi::VSPLUGINAPI,
) {
    ((*vspapi).configPlugin)(
        cstr!("com.example.invert").as_ptr(),
        cstr!("invert").as_ptr(),
        cstr!("VapourSynth Invert Example").as_ptr(),
        ffi::VS_MAKE_VERSION(1, 0),
        ffi::VAPOURSYNTH_API_VERSION,
        0,
        plugin,
    );

    ((*vspapi).registerFunction)(
        cstr!("Filter").as_ptr(),
        cstr!("clip:vnode;enabled:int:opt;").as_ptr(),
        cstr!("clip:vnode;").as_ptr(),
        filter_create,
        null_mut(),
        plugin,
    );
}

// declare_plugin!(
//     "com.example.filter",
//     "filter",
//     "VapourSynth Filter Skeleton",
//     ffi::VS_MAKE_VERSION(1, 0),
//     ffi::VAPOURSYNTH_API_VERSION,
//     0
// );

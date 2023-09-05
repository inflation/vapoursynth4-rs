use std::{
    ffi::{c_int, c_void, CString},
    ptr::{null, null_mut},
};

use const_str::cstr;
use vapoursynth4_rs::{key, ApiRef, CoreRef, Filter, FrameContext, MapMut, MapRef, NodeRef, Value};
use vapoursynth4_sys as ffi;

struct DumbFilter {
    node: NodeRef,
}

impl Filter for DumbFilter {
    type InstanceData = DumbFilter;

    fn get_frame(&self) -> ffi::VSFilterGetFrame {
        filter_get_frame
    }

    fn free(&self) -> ffi::VSFilterFree {
        Some(filter_free)
    }

    fn filter_mode(&self) -> vapoursynth4_rs::FilterMode {
        vapoursynth4_rs::FilterMode::Parallel
    }

    fn instance_data(&mut self) -> *mut Self::InstanceData {
        self
    }
}

/// # Safety
pub unsafe extern "system" fn filter_get_frame(
    n: c_int,
    activation_reason: ffi::VSActivationReason,
    instance_data: *mut c_void,
    frame_data: *mut *mut c_void,
    frame_ctx: *mut ffi::VSFrameContext,
    core: *mut ffi::VSCore,
    vsapi: *const ffi::VSAPI,
) -> *const ffi::VSFrame {
    use ffi::VSActivationReason as r;

    let d = &mut *(instance_data as *mut DumbFilter);
    let api = ApiRef::from_raw(vsapi);
    _ = api.set();
    let mut ctx = FrameContext::from_ptr(frame_ctx);

    match activation_reason {
        r::arInitial => ctx.request_frame_filter(n, &d.node),
        r::arAllFramesReady => {
            let mut core = CoreRef::from_ptr(core);
            let frame = ctx.get_frame_filter(n, &d.node);

            // TODO: do something with the frame
            core.log(ffi::VSMessageType::mtInformation, cstr!("Hello, world"));

            return frame.as_ptr();
        }
        r::arError => (api.setFilterError)(cstr!("Error").as_ptr(), frame_ctx),
    }

    null()
}

/// # Safety
pub unsafe extern "system" fn filter_free(
    instance_data: *mut c_void,
    core: *mut ffi::VSCore,
    vsapi: *const ffi::VSAPI,
) {
    // _ = Box::from_raw(instance_data as *mut DumbFilter);
}

/// # Safety
pub unsafe extern "system" fn filter_create(
    in_: *const ffi::VSMap,
    out: *mut ffi::VSMap,
    user_data: *mut c_void,
    core: *mut ffi::VSCore,
    vsapi: *const ffi::VSAPI,
) {
    let api = ApiRef::from_raw(vsapi);
    api.set().expect("API already set");
    let in_ = MapRef::from_ptr(in_);

    if let Err(e) = std::panic::catch_unwind(|| {
        let mut out = MapMut::from_ptr(out);
        let mut core = CoreRef::from_ptr(core);

        let node = match in_.get(key!("clip"), 0) {
            Ok(Value::VideoNode(node)) => node,
            _ => {
                out.set_error(cstr!("Failed to get node\n"));
                return;
            }
        };

        let mut filter = Box::new(DumbFilter { node: node.clone() });

        let deps = [ffi::VSFilterDependency {
            source: filter.node.as_mut_ptr(),
            requestPattern: ffi::VSRequestPattern::rpGeneral,
        }];
        let info = filter.node.get_video_info().clone();

        core.create_video_filter(out, "Filter", &info, filter, &deps)
            .unwrap();
    }) {
        let msg = CString::from_vec_unchecked(
            format!("{:?}", e.downcast_ref::<&str>().unwrap_unchecked()).into(),
        );
        MapMut::from_ptr(out).set_error(&msg);
    }
}

/// # Safety
#[no_mangle]
pub unsafe extern "system" fn VapourSynthPluginInit2(
    plugin: *mut ffi::VSPlugin,
    vspapi: *const ffi::VSPLUGINAPI,
) {
    ((*vspapi).configPlugin)(
        cstr!("com.example.filter").as_ptr(),
        cstr!("filter").as_ptr(),
        cstr!("VapourSynth Filter Skeleton").as_ptr(),
        ffi::VS_MAKE_VERSION(1, 0),
        ffi::VAPOURSYNTH_API_VERSION,
        0,
        plugin,
    );

    ((*vspapi).registerFunction)(
        cstr!("Filter").as_ptr(),
        cstr!("clip:vnode;").as_ptr(),
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

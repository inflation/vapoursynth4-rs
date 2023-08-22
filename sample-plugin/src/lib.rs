use std::{
    ffi::{c_int, c_void, CStr},
    ptr::{null, null_mut},
};

use vapoursynth4_rs::declare_plugin;
use vapoursynth4_sys as ffi;

struct FilterData {
    node: *mut ffi::VSNode,
    vi: *const ffi::VSVideoInfo,
}

/// # Safety
#[no_mangle]
pub unsafe extern "system" fn filterGetFrame(
    n: c_int,
    activation_reason: ffi::VSActivationReason,
    instance_data: *mut c_void,
    frame_data: *mut *mut c_void,
    frame_ctx: *mut ffi::VSFrameContext,
    core: *mut ffi::VSCore,
    vsapi: *const ffi::VSAPI,
) -> *const ffi::VSFrame {
    use ffi::VSActivationReason as r;

    let d = &mut *(instance_data as *mut FilterData);

    match activation_reason {
        r::arInitial => ((*vsapi).requestFrameFilter)(n, d.node, frame_ctx),
        r::arAllFramesReady => {
            let frame = ((*vsapi).getFrameFilter)(n, d.node, frame_ctx);

            // TODO: do something with the frame
            ((*vsapi).logMessage)(
                ffi::VSMessageType::mtInformation,
                CStr::from_bytes_with_nul_unchecked(b"Hello, world!\0").as_ptr(),
                core,
            );

            return frame;
        }
        r::arError => todo!(),
    }

    null()
}

/// # Safety
pub unsafe extern "system" fn filter_free(
    instance_data: *mut c_void,
    core: *mut ffi::VSCore,
    vsapi: *const ffi::VSAPI,
) {
    let d = Box::from_raw(instance_data as *mut FilterData);
    ((*vsapi).freeNode)(d.node);
}

/// # Safety
pub unsafe extern "system" fn filter_create(
    in_: *const ffi::VSMap,
    out: *mut ffi::VSMap,
    user_data: *mut c_void,
    core: *mut ffi::VSCore,
    vsapi: *const ffi::VSAPI,
) {
    let node = ((*vsapi).mapGetNode)(in_, b"clip\0".as_ptr() as *const _, 0, null_mut());

    let d = Box::new(FilterData {
        node,
        vi: ((*vsapi).getVideoInfo)(node),
    });

    let deps = [ffi::VSFilterDependency {
        source: d.node,
        requestPattern: ffi::VSRequestPattern::rpGeneral,
    }];

    ((*vsapi).createVideoFilter)(
        out,
        CStr::from_bytes_with_nul_unchecked(b"Filter\0").as_ptr(),
        d.vi,
        filterGetFrame,
        Some(filter_free),
        ffi::VSFilterMode::fmParallel,
        deps.as_ptr(),
        deps.len() as _,
        Box::into_raw(d).cast(),
        core,
    )
}

// /// # Safety
// #[no_mangle]
// pub unsafe extern "system" fn VapourSynthPluginInit2(
//     plugin: *mut ffi::VSPlugin,
//     vspapi: *const ffi::VSPLUGINAPI,
// ) {
//     ((*vspapi).configPlugin)(
//         CStr::from_bytes_with_nul_unchecked(b"com.example.filter\0").as_ptr(),
//         CStr::from_bytes_with_nul_unchecked(b"filter\0").as_ptr(),
//         CStr::from_bytes_with_nul_unchecked(b"VapourSynth Filter Skeleton\0").as_ptr(),
//         ffi::VS_MAKE_VERSION(1, 0),
//         ffi::VAPOURSYNTH_API_VERSION,
//         0,
//         plugin,
//     );

//     ((*vspapi).registerFunction)(
//         CStr::from_bytes_with_nul_unchecked(b"Filter\0").as_ptr(),
//         CStr::from_bytes_with_nul_unchecked(b"clip:vnode;\0").as_ptr(),
//         CStr::from_bytes_with_nul_unchecked(b"clip:vnode;\0").as_ptr(),
//         filter_create,
//         null_mut(),
//         plugin,
//     );
// }

declare_plugin!(
    "com.example.filter",
    "filter",
    "VapourSynth Filter Skeleton",
    ffi::VS_MAKE_VERSION(1, 0),
    ffi::VAPOURSYNTH_API_VERSION,
    0
);

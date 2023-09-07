use std::{
    ffi::{c_void, CString},
    panic::AssertUnwindSafe,
    ptr::null_mut,
};

use const_str::cstr;
use ffi::helper::isConstantVideoFormat;
use vapoursynth4_rs::{
    key, ApiRef, CoreRef, Filter, FilterExtern, FrameContext, MapMut, MapRef, Node, ToCString,
    Value, VideoFrame, VideoNode,
};
use vapoursynth4_sys as ffi;

struct DumbFilter {
    node: VideoNode,
    enabled: bool,
}

impl Filter for DumbFilter {
    type Error = CString;
    type FrameType = VideoFrame;

    fn create<T>(
        input: MapRef<'_>,
        output: MapMut<'_>,
        _data: Option<&mut T>,
        mut core: CoreRef,
    ) -> Result<(), Self::Error> {
        let node = input
            .get_video_node(key!("clip"), 0)
            .expect("Failed to get clip");
        let vi = node.get_info();

        unsafe {
            if !isConstantVideoFormat(&*vi)
                || (*vi).format.sampleType != ffi::VSSampleType::stInteger
                || (*vi).format.bitsPerSample != 8
            {
                panic!("Invert: only constant format 8bit integer input supported");
            }
        }

        let mut filter = DumbFilter {
            node,
            enabled: input
                .get_int(key!("enabled"), 0)
                .map(|v| v != 0)
                .unwrap_or(true),
        };

        let deps = [ffi::VSFilterDependency {
            source: filter.node.as_mut_ptr(),
            requestPattern: ffi::VSRequestPattern::rpStrictSpatial,
        }];

        unsafe { core.create_video_filter(output, "Invert", &*vi, Box::new(filter), &deps) }
            .map_err(ToCString::into_cstring_lossy)?;

        Ok(())
    }

    fn get_frame(
        &self,
        n: i32,
        activation_reason: ffi::VSActivationReason,
        _frame_data: *mut *mut c_void,
        mut ctx: FrameContext,
        core: CoreRef,
    ) -> Result<Option<VideoFrame>, Self::Error> {
        use ffi::VSActivationReason as r;

        match activation_reason {
            r::arInitial => {
                ctx.request_frame_filter(n, &self.node);
            }
            r::arAllFramesReady => {
                let src = self.node.get_frame_filter(n, &mut ctx);
                if !self.enabled {
                    panic!("Not enabled");
                }

                let fi = src.get_video_format();
                let height = src.frame_height(0);
                let width = src.frame_width(0);

                let mut dst = core.new_video_frame(fi, width, height, Some(&src));

                for plane in 0..fi.numPlanes {
                    let mut src_p = src.plane(plane);
                    let src_stride = src.stride(plane);
                    let mut dst_p = dst.plane_mut(plane);
                    let dst_stride = dst.stride(plane);

                    let h = src.frame_height(plane);
                    let w = src.frame_width(plane);

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
        DumbFilter::filter_create,
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

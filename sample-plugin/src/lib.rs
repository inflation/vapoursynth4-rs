mod dither;

use std::ffi::{c_void, CStr};

use const_str::cstr;
use dither::DitherFilter;
use vapoursynth4_rs::{
    core::CoreRef,
    declare_plugin,
    frame::{FrameContext, VideoFrame},
    key,
    map::MapRef,
    node::{
        ActivationReason, Dependencies, Filter, FilterDependency, Node, RequestPattern, VideoNode,
    },
    SampleType,
};

struct DumbFilter {
    node: VideoNode,
    enabled: bool,
}

impl Filter for DumbFilter {
    type Error = &'static CStr;
    type FrameType = VideoFrame;
    type FilterData = ();

    fn create(
        input: &MapRef,
        output: &mut MapRef,
        _data: Option<Box<Self::FilterData>>,
        mut core: CoreRef,
    ) -> Result<(), Self::Error> {
        let Ok(node) = input.get_video_node(key!("clip"), 0) else {
            return Err(cstr!("Failed to get clip"));
        };
        let n = node.clone();
        let vi = n.info();

        if !vapoursynth4_rs::utils::is_constant_video_format(vi)
            || vi.format.sample_type != SampleType::Integer
            || vi.format.bits_per_sample != 8
        {
            return Err(cstr!(
                "Invert: only constant format 8bit integer input supported"
            ));
        }

        let mut filter = DumbFilter {
            node,
            enabled: input
                .get_int(key!("enabled"), 0)
                .map(|v| v != 0)
                .unwrap_or(true),
        };

        let deps = [FilterDependency {
            source: filter.node.as_mut_ptr(),
            request_pattern: RequestPattern::StrictSpatial,
        }];

        core.create_video_filter(
            output,
            cstr!("Invert"),
            vi,
            Box::new(filter),
            Dependencies::new(&deps).unwrap(),
        );

        Ok(())
    }

    fn get_frame(
        &self,
        n: i32,
        activation_reason: ActivationReason,
        _frame_data: *mut *mut c_void,
        mut ctx: FrameContext,
        core: CoreRef,
    ) -> Result<Option<VideoFrame>, Self::Error> {
        use ActivationReason as r;

        match activation_reason {
            r::Initial => {
                ctx.request_frame_filter(n, &self.node);
            }
            r::AllFramesReady => {
                let src = self.node.get_frame_filter(n, &mut ctx);
                if !self.enabled {
                    panic!("Not enabled");
                }

                let fi = src.get_video_format();
                let height = src.frame_height(0);
                let width = src.frame_width(0);

                let mut dst = core.new_video_frame(fi, width, height, Some(&src));

                for plane in 0..fi.num_planes {
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

    const NAME: &'static CStr = cstr!("Filter");
    const ARGS: &'static CStr = cstr!("clip:vnode;enabled:int:opt;");
    const RETURN_TYPE: &'static CStr = cstr!("clip:vnode;");
}

declare_plugin!(
    "com.example.invert",
    "invert",
    "VapourSynth Filter Skeleton",
    (1, 0),
    vapoursynth4_rs::VAPOURSYNTH_API_VERSION,
    0,
    (DumbFilter, None),
    (DitherFilter, None)
);

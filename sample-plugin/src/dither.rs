use std::ffi::{c_void, CStr};

use const_str::cstr;
use vapoursynth4_rs::{
    core::CoreRef,
    frame::{FrameContext, VideoFrame},
    key,
    map::{AppendMode, Map, MapRef, Value},
    node::{
        ActivationReason, Dependencies, Filter, FilterDependency, Node, RequestPattern, VideoNode,
    },
};

/// An example filter that dithers the input clip to the specified bit depth
/// using the fmtconv plugin. This demonstrates how to invoke other plugins.
pub(crate) struct DitherFilter {
    /// Input node.
    node: VideoNode,
}

impl Filter for DitherFilter {
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

        // Input parameters.
        let bits = input.get_int_saturated(key!("bits"), 0).unwrap_or(16);

        // Use fmtconv to dither to the desired bit depth.
        let Some(fmtc_plugin) = core.get_plugin_by_namespace(cstr!("fmtc")) else {
            return Err(cstr!("Failed to find the fmtconv plugin."));
        };
        let mut args = Map::new();
        args.set(
            key!("clip"),
            Value::VideoNode(node.clone()),
            AppendMode::Replace,
        )
        .unwrap();
        args.set(key!("bits"), Value::Int(bits as i64), AppendMode::Replace)
            .unwrap();
        args.set(key!("dmode"), Value::Int(8), AppendMode::Replace)
            .unwrap();
        let ret = fmtc_plugin.invoke(cstr!("bitdepth"), &args);
        let Ok(dithered_node) = ret.get_video_node(key!("clip"), 0) else {
            return Err(cstr!("Failed to dither the clip."));
        };

        // Update output info to reflect the new bit depth.
        let mut vi = node.info().clone();
        vi.format = core.query_video_format(
            vi.format.color_family,
            vi.format.sample_type,
            bits,
            vi.format.sub_sampling_w,
            vi.format.sub_sampling_h,
        );

        let mut filter = DitherFilter {
            node: dithered_node,
        };

        let deps = [FilterDependency {
            source: filter.node.as_mut_ptr(),
            request_pattern: RequestPattern::StrictSpatial,
        }];

        core.create_video_filter(
            output,
            cstr!("Depth"),
            &vi,
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
                let dst = core.copy_frame(&src);

                // Do whatever frame processing here, in the new bit depth.

                return Ok(Some(dst));
            }
            _ => {}
        }

        Ok(None)
    }

    const NAME: &'static CStr = cstr!("Depth");
    const ARGS: &'static CStr = cstr!("clip:vnode;bits:int:opt;");
    const RETURN_TYPE: &'static CStr = cstr!("clip:vnode;");
}
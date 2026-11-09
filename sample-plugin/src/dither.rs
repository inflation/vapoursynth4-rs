use std::ffi::{CStr, c_void};

use vapoursynth4_rs::{
    core::CoreRef,
    frame::{FrameContext, VideoFrame},
    key,
    map::{AppendMode, MapMut, MapRef, Value},
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
        input: MapRef,
        mut output: MapMut,
        _data: Option<&Self::FilterData>,
        core: CoreRef,
    ) -> Result<(), Self::Error> {
        let Ok(node) = input.get_video_node(key!(c"clip"), 0) else {
            return Err(c"Failed to get clip");
        };

        // Input parameters.
        let bits = input.get_int_saturated(key!(c"bits"), 0).unwrap_or(16);

        // Use fmtconv to dither to the desired bit depth.
        let Some(fmtc_plugin) = core.get_plugin_by_namespace(c"fmtc") else {
            return Err(c"Failed to find the fmtconv plugin.");
        };
        let mut args = core.create_map();
        args.set(
            key!(c"clip"),
            Value::VideoNode(node.clone()),
            AppendMode::Replace,
        )
        .unwrap();
        args.set(key!(c"bits"), Value::Int(bits as i64), AppendMode::Replace)
            .unwrap();
        args.set(key!(c"dmode"), Value::Int(8), AppendMode::Replace)
            .unwrap();
        let ret = fmtc_plugin.invoke(c"bitdepth", &args);
        let Ok(dithered_node) = ret.get_video_node(key!(c"clip"), 0) else {
            return Err(c"Failed to dither the clip.");
        };

        // Update output info to reflect the new bit depth.
        let mut vi = node.info().clone();
        vi.format = core
            .query_video_format(
                vi.format.color_family,
                vi.format.sample_type,
                bits,
                vi.format.sub_sampling_w,
                vi.format.sub_sampling_h,
            )
            .ok_or(c"Unsupported output format")?;

        let filter = DitherFilter {
            node: dithered_node,
        };

        let deps = [FilterDependency {
            source: filter.node.as_ptr(),
            request_pattern: RequestPattern::StrictSpatial,
        }];

        core.create_video_filter(
            &mut output,
            c"Depth",
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

                return Ok(Some(dst.freeze()));
            }
            _ => {}
        }

        Ok(None)
    }

    const NAME: &'static CStr = c"Depth";
    const ARGS: &'static CStr = c"clip:vnode;bits:int:opt;";
    const RETURN_TYPE: &'static CStr = c"clip:vnode;";
}

#[cfg(test)]
mod tests {
    use super::*;
    use vapoursynth4_rs::{core::Core, sciprt::Script};

    #[test]
    fn reports_missing_input_and_missing_fmtconv() -> testresult::TestResult {
        let script = Script::new(
            Some(Core::builder().disable_auto_loading().build()),
            Default::default(),
        );
        let core = script.core()?;
        let mut output = core.create_map();
        let args = core.create_map();
        assert_eq!(
            DitherFilter::create(args.as_ref(), output.as_mut(), None, script.core()?),
            Err(c"Failed to get clip")
        );

        let std = core.get_plugin_by_namespace(c"std").unwrap();
        let clip = std.invoke(c"BlankClip", args);
        assert_eq!(
            DitherFilter::create(clip.as_ref(), output.as_mut(), None, script.core()?),
            Err(c"Failed to find the fmtconv plugin.")
        );
        Ok(())
    }

    #[test]
    fn copies_frames_from_the_dithered_clip() -> testresult::TestResult {
        let script = Script::default();
        let core = script.core()?;
        let std = core.get_plugin_by_namespace(c"std").unwrap();
        let clip = std.invoke(c"BlankClip", core.create_map());
        let source = clip.get_video_node(key!(c"clip"), 0)?;
        let info = source.info().clone();
        let deps = [FilterDependency {
            source: source.as_ptr(),
            request_pattern: RequestPattern::StrictSpatial,
        }];
        let mut output = core.create_map();
        core.create_video_filter(
            &mut output.as_mut(),
            c"Depth",
            &info,
            Box::new(DitherFilter { node: source }),
            Dependencies::new(&deps).unwrap(),
        );
        let filtered = output.get_video_node(key!(c"clip"), 0)?;
        let frame = filtered
            .get_frame(0)
            .map_err(|error| error.to_string_lossy().into_owned())?;
        assert_eq!(frame.frame_width(0), info.width);
        assert_eq!(
            frame.get_video_format().bits_per_sample,
            info.format.bits_per_sample
        );
        Ok(())
    }
}

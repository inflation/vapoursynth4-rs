/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::ffi::{c_void, CStr};

use vapoursynth4_rs::{
    core::CoreRef,
    filter_name,
    frame::{FrameContext, FrameTypeVideo, VideoFrame},
    key,
    map::{AppendMode, MapRef},
    node::{
        ActivationReason, Dependencies, Filter, FilterDependency, FilterName, RequestPattern,
        VideoNode,
    },
};

pub struct SpliceFilter {
    nodes: Vec<VideoNode>,
    num_frames: Vec<i32>,
}

impl Filter for SpliceFilter {
    type Error = &'static CStr;
    type FrameType = FrameTypeVideo;
    type FilterData = ();

    fn create(
        input: MapRef,
        output: MapRef,
        _data: Option<&Self::FilterData>,
        core: CoreRef,
    ) -> Result<(), Self::Error> {
        let num_clips = input.num_elements(key!("clips")).unwrap();

        if num_clips == 1 {
            output.consume_node(
                key!("clip"),
                input.get_video_node(key!("clips"), 0).unwrap(),
                AppendMode::Replace,
            )?;
        } else {
            let nodes = (0..num_clips)
                .map(|i| input.get_video_node(key!("clips"), i))
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            let (num_frames, total_frames) = nodes
                .iter()
                .map(|n| n.info().num_frames)
                .try_fold((vec![], 0), |(mut v, mut n), x| {
                    v.push(x);
                    n += x;

                    if n < x {
                        Err(c"Splice: the resulting clip is too long")
                    } else {
                        Ok((v, n))
                    }
                })
                .unwrap();

            let deps: Vec<_> = nodes
                .iter()
                .map(|n| FilterDependency {
                    source: n.as_ptr(),
                    request_pattern: RequestPattern::NoFrameReuse,
                })
                .collect();

            let mut vi = nodes[0].info().clone();
            vi.num_frames = total_frames;

            let filter = SpliceFilter { nodes, num_frames };

            core.create_video_filter(
                output,
                c"Splice",
                &vi,
                Box::new(filter),
                Dependencies::new(&deps).unwrap(),
            );
        }

        Ok(())
    }

    fn get_frame(
        &self,
        n: i32,
        activation_reason: ActivationReason,
        frame_data: &mut [*mut c_void; 4],
        ctx: FrameContext,
        _core: CoreRef,
    ) -> Result<Option<VideoFrame>, Self::Error> {
        use ActivationReason as r;

        match activation_reason {
            r::Initial => {
                let mut frame = 0;
                let mut idx = 0;
                let mut cum_frame = 0;

                for (i, x) in self.num_frames.iter().enumerate() {
                    if (n >= cum_frame && n < cum_frame + x) || i == (self.num_frames.len() - 1) {
                        idx = i;
                        frame = n - cum_frame;
                        break;
                    }

                    cum_frame += x;
                }

                frame_data[0] = Box::into_raw(Box::new(self.nodes[idx].clone())).cast();
                frame_data[1] = frame as *mut c_void;

                self.nodes[idx].request_frame_filter(frame, &ctx);
            }
            r::AllFramesReady => {
                // MUST drop any frame data when the frame is ready
                return Ok(Some(unsafe {
                    (Box::from_raw(frame_data[0].cast::<VideoNode>()))
                        .get_frame_filter(frame_data[1] as i32, &ctx)
                }));
            }
            r::Error => return Err(c"Aborted"),
        }

        Ok(None)
    }

    const NAME: &'static FilterName = filter_name!("Splice");
    const ARGS: &'static CStr = c"clips:vnode[];";
    const RETURN_TYPE: &'static CStr = c"clip:vnode;";
}

/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

mod dependency;
mod filter;
pub(crate) mod internal;

use std::ffi::{CStr, CString, c_void};

use crate::{
    AudioInfo, VideoInfo,
    api::Api,
    core::CoreHandle,
    ffi,
    frame::{AudioFrame, Frame, FrameContext, VideoFrame, internal::FrameFromPtr},
    node::internal::FilterExtern,
};

pub use dependency::*;
pub use filter::*;
use vapoursynth4_sys::VSFrameDoneCallback;

pub trait Node: Sized + Send + Sync + crate::_private::Sealed {
    type FrameType: Frame;

    fn api(&self) -> Api;

    #[must_use]
    fn as_ptr(&self) -> *mut ffi::VSNode;

    #[must_use]
    fn get_frame_filter(&self, n: i32, ctx: &mut FrameContext) -> Self::FrameType;

    /// Returns whether the data resides in host or GPU memory.
    #[cfg(feature = "vs-43")]
    #[must_use]
    fn residency(&self) -> ffi::VSNodeResidency {
        unsafe { (self.api().getNodeResidency)(self.as_ptr()) }
    }

    fn set_linear_filter(&mut self) -> i32 {
        unsafe { (self.api().setLinearFilter)(self.as_ptr()) }
    }

    fn set_cache_mode(&mut self, mode: CacheMode) {
        unsafe {
            (self.api().setCacheMode)(self.as_ptr(), mode);
        }
    }

    fn set_cache_options(&mut self, fixed_size: i32, max_size: i32, max_history_size: i32) {
        unsafe {
            (self.api().setCacheOptions)(self.as_ptr(), fixed_size, max_size, max_history_size);
        }
    }

    /// # Errors
    ///
    /// Return the internal error message if the frame is not ready.
    fn get_frame(&self, n: i32) -> Result<Self::FrameType, CString> {
        let mut buf = [0; 1024];
        let ptr = unsafe { (self.api().getFrame)(n, self.as_ptr(), buf.as_mut_ptr(), 1024) };

        if ptr.is_null() {
            Err(unsafe { CStr::from_ptr(buf.as_ptr()).into() })
        } else {
            unsafe { Ok(Self::FrameType::from_ptr(ptr, self.api())) }
        }
    }

    // TODO: Find a better way to handle callbacks
    /// # Safety
    ///
    /// The caller must ensure that:
    /// - `data` is a valid pointer to the data needed by the callback
    /// - `callback` is a valid function pointer that safely handles the frame data
    /// - The callback and data remain valid until the frame processing is complete
    unsafe fn get_frame_async(&self, n: i32, data: *mut c_void, callback: VSFrameDoneCallback) {
        unsafe {
            (self.api().getFrameAsync)(n, self.as_ptr(), callback, data);
        }
    }

    /// Name of the function that created this filter.
    ///
    /// `level` 0 is the creating function itself; higher levels walk up to the
    /// function that invoked it. Returns [`None`] for a non-existent level.
    ///
    /// Only meaningful on a core built with
    /// [`enable_graph_inspection()`](crate::core::CoreBuilder::enable_graph_inspection),
    /// and not safe to call concurrently with frame requests.
    #[cfg(feature = "vs-graph")]
    #[must_use]
    fn creation_function_name(&self, level: i32) -> Option<&CStr> {
        unsafe {
            to_cstr((self.api().getNodeCreationFunctionName)(
                self.as_ptr(),
                level,
            ))
        }
    }

    /// Identifier of the plugin that created this filter.
    ///
    /// See [`creation_function_name()`](Self::creation_function_name) for `level`
    /// and for the constraints on when this may be called.
    #[cfg(feature = "vs-graph")]
    #[must_use]
    fn creation_plugin_id(&self, level: i32) -> Option<&CStr> {
        unsafe { to_cstr((self.api().getNodeCreationPluginID)(self.as_ptr(), level)) }
    }

    /// Namespace of the plugin that created this filter.
    ///
    /// See [`creation_function_name()`](Self::creation_function_name) for `level`
    /// and for the constraints on when this may be called.
    #[cfg(feature = "vs-graph")]
    #[must_use]
    fn creation_plugin_namespace(&self, level: i32) -> Option<&CStr> {
        unsafe { to_cstr((self.api().getNodeCreationPluginNS)(self.as_ptr(), level)) }
    }

    /// Arguments passed to the function that created this filter.
    ///
    /// See [`creation_function_name()`](Self::creation_function_name) for `level`
    /// and for the constraints on when this may be called.
    #[cfg(feature = "vs-graph")]
    #[must_use]
    fn creation_function_arguments(&self, level: i32) -> Option<crate::map::MapRef<'_>> {
        let ptr = unsafe { (self.api().getNodeCreationFunctionArguments)(self.as_ptr(), level) };
        (!ptr.is_null()).then(|| unsafe { crate::map::MapRef::from_ptr(ptr, self.api()) })
    }
}

/// The graph inspection functions return strings owned by the node, so the
/// result borrows from `self` rather than being copied out.
#[cfg(feature = "vs-graph")]
unsafe fn to_cstr<'a>(ptr: *const std::ffi::c_char) -> Option<&'a CStr> {
    (!ptr.is_null()).then(|| unsafe { CStr::from_ptr(ptr) })
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct VideoNode {
    handle: *const ffi::VSNode,
    api: Api,
}

impl crate::_private::Sealed for VideoNode {}
unsafe impl Send for VideoNode {}
unsafe impl Sync for VideoNode {}

impl Node for VideoNode {
    type FrameType = VideoFrame;

    #[inline]
    fn api(&self) -> Api {
        self.api
    }

    #[inline]
    fn as_ptr(&self) -> *mut ffi::VSNode {
        self.handle.cast_mut()
    }

    fn get_frame_filter(&self, n: i32, ctx: &mut FrameContext) -> Self::FrameType {
        unsafe {
            VideoFrame::from_ptr(
                (self.api.getFrameFilter)(n, self.as_ptr(), ctx.as_ptr()),
                self.api,
            )
        }
    }
}

impl VideoNode {
    /// # Safety
    ///
    /// The caller must ensure that `ptr` is a valid pointer to a [`ffi::VSNode`] that represents a video node.
    #[must_use]
    pub unsafe fn from_ptr(ptr: *mut ffi::VSNode, api: Api) -> Self {
        Self { handle: ptr, api }
    }

    #[must_use]
    pub fn info(&self) -> &VideoInfo {
        // SAFETY: `vi` is valid if the node is a video node
        unsafe { &*(self.api.getVideoInfo)(self.as_ptr()) }
    }

    /// # Panics
    ///
    /// Panics if the the dependency index is larger than [`i32::MAX`].
    pub fn new<F: Filter>(
        name: &str,
        info: &VideoInfo,
        filter: F,
        dependencies: &[ffi::VSFilterDependency],
        core: &CoreHandle,
    ) -> Option<Self> {
        let filter = Box::new(filter);
        let name = CString::new(name).ok()?;
        let create = core::cfg_select! {
            feature = "vs-43" => core.api().createVideoFilterEx2,
            _ => core.api().createVideoFilter2,
        };
        let ptr = unsafe {
            create(
                name.as_ptr(),
                info,
                F::filter_get_frame,
                Some(F::filter_free),
                F::FILTER_MODE,
                #[cfg(feature = "vs-43")]
                F::FLAGS,
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                Box::into_raw(filter).cast(),
                core.as_ptr(),
            )
        };
        (!ptr.is_null()).then(|| unsafe { Self::from_ptr(ptr, core.api()) })
    }
}

impl Clone for VideoNode {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((self.api.addNodeRef)(self.as_ptr()), self.api) }
    }
}

impl Drop for VideoNode {
    fn drop(&mut self) {
        unsafe { (self.api.freeNode)(self.as_ptr()) }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct AudioNode {
    handle: *const ffi::VSNode,
    api: Api,
}

impl crate::_private::Sealed for AudioNode {}
unsafe impl Send for AudioNode {}
unsafe impl Sync for AudioNode {}

impl Node for AudioNode {
    type FrameType = AudioFrame;

    #[inline]
    fn api(&self) -> Api {
        self.api
    }

    #[inline]
    fn as_ptr(&self) -> *mut ffi::VSNode {
        self.handle.cast_mut()
    }

    fn get_frame_filter(&self, n: i32, ctx: &mut FrameContext) -> Self::FrameType {
        unsafe {
            AudioFrame::from_ptr(
                (self.api.getFrameFilter)(n, self.as_ptr(), ctx.as_ptr()),
                self.api,
            )
        }
    }
}

impl AudioNode {
    /// # Safety
    ///
    /// The caller must ensure that `ptr` is a valid pointer to a [`ffi::VSNode`] that represents an audio node.
    #[must_use]
    pub unsafe fn from_ptr(ptr: *mut ffi::VSNode, api: Api) -> Self {
        Self { handle: ptr, api }
    }

    #[must_use]
    pub fn info(&self) -> &AudioInfo {
        // SAFETY: `ai` is valid if the node is an audio node
        unsafe { &*(self.api.getAudioInfo)(self.as_ptr()) }
    }

    /// # Panics
    ///
    /// Panics if the the dependency index is larger than [`i32::MAX`].
    pub fn new<F: Filter>(
        name: &str,
        info: &AudioInfo,
        filter: F,
        dependencies: &[ffi::VSFilterDependency],
        core: &CoreHandle,
    ) -> Option<Self> {
        let filter = Box::new(filter);
        let name = CString::new(name).ok()?;
        let ptr = unsafe {
            (core.api().createAudioFilter2)(
                name.as_ptr(),
                info,
                F::filter_get_frame,
                Some(F::filter_free),
                F::FILTER_MODE,
                dependencies.as_ptr(),
                dependencies.len().try_into().unwrap(),
                Box::into_raw(filter).cast(),
                core.as_ptr(),
            )
        };
        ptr.is_null()
            .then_some(unsafe { Self::from_ptr(ptr, core.api()) })
    }
}

impl Clone for AudioNode {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((self.api.addNodeRef)(self.as_ptr()), self.api) }
    }
}

impl Drop for AudioNode {
    fn drop(&mut self) {
        unsafe { (self.api.freeNode)(self.as_ptr()) }
    }
}

pub type FilterMode = ffi::VSFilterMode;
pub type CacheMode = ffi::VSCacheMode;

#[cfg(all(test, feature = "link-vs"))]
mod tests {
    use std::{ffi::c_char, sync::mpsc, time::Duration};

    use super::*;
    use crate::core::Core;

    type FrameResult = Result<VideoFrame, CString>;

    unsafe extern "system-unwind" fn complete(
        data: *mut c_void,
        frame: *const ffi::VSFrame,
        _: i32,
        _: *mut ffi::VSNode,
        error: *const c_char,
    ) {
        unsafe {
            let sender = Box::from_raw(data.cast::<mpsc::Sender<FrameResult>>());
            let result = if frame.is_null() {
                Err(CStr::from_ptr(error).into())
            } else {
                Ok(VideoFrame::from_ptr(frame, Api::default()))
            };
            let _ = sender.send(result);
        }
    }

    #[test]
    fn async_frame_requests_deliver_success_and_failure() -> testresult::TestResult {
        let core = Core::builder().disable_auto_loading().build();
        let plugin = core.get_plugin_by_namespace(c"std").unwrap();
        let output = plugin.invoke(c"BlankClip", core.create_map());
        let mut node = output.get_video_node(crate::key!(c"clip"), 0)?;
        node.set_cache_mode(CacheMode::ForceEnable);
        node.set_cache_options(0, 4, 2);
        node.set_linear_filter();

        for n in [0, node.info().num_frames] {
            let (sender, receiver) = mpsc::channel::<FrameResult>();
            unsafe {
                node.get_frame_async(n, Box::into_raw(Box::new(sender)).cast(), complete);
            }
            let result = receiver.recv_timeout(Duration::from_secs(10))?;
            if n == 0 {
                assert_eq!(result.unwrap().frame_width(0), node.info().width);
            } else {
                assert_ne!(result.unwrap_err().as_bytes(), b"");
            }
        }
        Ok(())
    }
}

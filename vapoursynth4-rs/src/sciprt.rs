use std::{
    ffi::{CStr, c_int},
    ptr::{NonNull, null_mut},
};

use thiserror::Error;
use vapoursynth4_sys::VAPOURSYNTH_API_VERSION;

use crate::{
    api::{Api, VssApi},
    core::{Core, CoreRef},
    node::{AudioNode, VideoNode},
};

use super::ffi;
#[cfg(feature = "vsscript-44")]
use crate::map::Map;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Script {
    handle: NonNull<ffi::VSScript>,
    vssapi: VssApi,
    api: Api,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OutputNode {
    Audio(AudioNode),
    Video(VideoNode),
}

impl OutputNode {
    #[must_use]
    pub fn as_audio(&self) -> Option<&AudioNode> {
        if let OutputNode::Audio(node) = self {
            Some(node)
        } else {
            None
        }
    }

    #[must_use]
    pub fn as_video(&self) -> Option<&VideoNode> {
        if let OutputNode::Video(node) = self {
            Some(node)
        } else {
            None
        }
    }
}

impl Script {
    /// Creates a new script instance.
    ///
    /// # Panics
    ///
    /// Panics if the script creation fails.
    pub fn new(core: Option<&Core>, vssapi: VssApi) -> Self {
        unsafe {
            let handle = NonNull::new((vssapi.createScript)(core.map_or(null_mut(), Core::as_ptr)))
                .expect("Failed to create script");
            let api_ptr = (vssapi.getVSAPI)(VAPOURSYNTH_API_VERSION);
            assert!(!api_ptr.is_null());
            let api = Api::from_ptr(api_ptr);
            Self {
                handle,
                vssapi,
                api,
            }
        }
    }

    #[must_use]
    pub fn get_api(&self) -> Api {
        self.api
    }

    /// Returns a reference to the core associated with this script.
    ///
    /// # Errors
    ///
    /// Returns an error message if the core could not be retrieved.
    pub fn core(&self) -> Result<CoreRef<'_>, ScriptError> {
        unsafe {
            let core = (self.vssapi.getCore)(self.handle.as_ptr());
            self.get_ptr_error(core)
                .map(|core| CoreRef::from_ptr(core, self.api))
        }
    }

    /// Evaluates a script buffer with the given filename.
    ///
    /// # Errors
    ///
    /// Returns an error message if the script evaluation fails.
    pub fn evaluate(&self, buffer: &CStr, filename: &CStr) -> Result<(), ScriptError> {
        unsafe {
            let result = (self.vssapi.evaluateBuffer)(
                self.handle.as_ptr(),
                buffer.as_ptr(),
                filename.as_ptr(),
            );
            self.get_error(result)
        }
    }

    /// Evaluates a script from a file.
    ///
    /// # Errors
    ///
    /// Returns an error message if the script evaluation fails.
    pub fn evaluate_file(&self, filename: &CStr) -> Result<(), ScriptError> {
        unsafe {
            let result = (self.vssapi.evaluateFile)(self.handle.as_ptr(), filename.as_ptr());
            self.get_error(result)
        }
    }

    /// Gets the output node at the specified index.
    ///
    /// # Errors
    ///
    /// Returns a `ScriptError` if the output node could not be retrieved.
    pub fn get_output(&self, index: c_int) -> Result<OutputNode, ScriptError> {
        unsafe {
            let ptr =
                self.get_ptr_error((self.vssapi.getOutputNode)(self.handle.as_ptr(), index))?;
            Ok(self.output_from_ptr(ptr))
        }
    }

    /// Gets an output node, using a bitmask of [`ffi::VSScriptGetFlags`].
    /// Pass 0 to download GPU output or `AllowGPUResident` to preserve residency.
    /// Release the returned node before this script.
    ///
    /// # Errors
    ///
    /// Returns a `ScriptError` if the output is unset or could not be retrieved.
    #[cfg(feature = "vsscript-44")]
    pub fn get_output_ex(&self, index: c_int, flags: c_int) -> Result<OutputNode, ScriptError> {
        unsafe {
            let ptr = self.get_ptr_error((self.vssapi.getOutputNodeEx)(
                self.handle.as_ptr(),
                index,
                flags,
            ))?;
            Ok(self.output_from_ptr(ptr))
        }
    }

    /// Gets an alpha output using a bitmask of [`ffi::VSScriptGetFlags`].
    /// Pass 0 to download GPU output or `AllowGPUResident` to preserve residency.
    /// Release the returned node before this script.
    ///
    /// # Errors
    ///
    /// Returns a `ScriptError` if the alpha output is unset or unavailable.
    #[cfg(feature = "vsscript-44")]
    pub fn get_output_alpha_ex(
        &self,
        index: c_int,
        flags: c_int,
    ) -> Result<VideoNode, ScriptError> {
        unsafe {
            let ptr = self.get_ptr_error((self.vssapi.getOutputAlphaNodeEx)(
                self.handle.as_ptr(),
                index,
                flags,
            ))?;
            Ok(VideoNode::from_ptr(ptr, self.api))
        }
    }

    /// Lists output indices using a bitmask of [`ffi::VSScriptGetFlags`].
    ///
    /// # Errors
    ///
    /// Returns a `ScriptError` if output enumeration fails.
    #[cfg(feature = "vsscript-44")]
    pub fn get_available_outputs_ex(&self, flags: c_int) -> Result<Vec<c_int>, ScriptError> {
        let count = unsafe {
            (self.vssapi.getAvailableOutputNodesEx)(self.handle.as_ptr(), 0, null_mut(), flags)
        };
        let len = usize::try_from(count).map_err(|_| unsafe { ScriptError::from_vss(self) })?;
        let mut outputs = vec![0; len];
        let count = unsafe {
            (self.vssapi.getAvailableOutputNodesEx)(
                self.handle.as_ptr(),
                count,
                outputs.as_mut_ptr(),
                flags,
            )
        };
        outputs
            .truncate(usize::try_from(count).map_err(|_| unsafe { ScriptError::from_vss(self) })?);
        Ok(outputs)
    }

    /// Copies a script variable into `dst` under its name.
    /// `flags` is a bitmask of [`ffi::VSScriptGetFlags`]. With no flags GPU nodes
    /// are downloaded and GPU frame variables are rejected. `AllowGPUResident`
    /// preserves the residency of nodes and frames.
    ///
    /// # Errors
    ///
    /// Returns a `ScriptError` if the variable is missing or cannot be stored.
    #[cfg(feature = "vsscript-44")]
    pub fn get_variable_ex(
        &self,
        name: &CStr,
        dst: &mut Map,
        flags: c_int,
    ) -> Result<(), ScriptError> {
        let result = unsafe {
            (self.vssapi.getVariableEx)(self.handle.as_ptr(), name.as_ptr(), dst.as_ptr(), flags)
        };
        self.get_error(result)
    }
}

// MARK: Helper
impl Script {
    unsafe fn output_from_ptr(&self, ptr: *mut ffi::VSNode) -> OutputNode {
        unsafe {
            match (self.api.getNodeType)(ptr) {
                ffi::VSMediaType::Audio => OutputNode::Audio(AudioNode::from_ptr(ptr, self.api)),
                ffi::VSMediaType::Video => OutputNode::Video(VideoNode::from_ptr(ptr, self.api)),
            }
        }
    }

    fn get_error(&self, ret: c_int) -> Result<(), ScriptError> {
        if ret == 0 {
            Ok(())
        } else {
            Err(unsafe { ScriptError::from_vss(self) })
        }
    }

    fn get_ptr_error<T>(&self, ptr: *mut T) -> Result<*mut T, ScriptError> {
        if ptr.is_null() {
            Err(unsafe { ScriptError::from_vss(self) })
        } else {
            Ok(ptr)
        }
    }
}

impl Drop for Script {
    fn drop(&mut self) {
        unsafe { (self.vssapi.freeScript)(self.handle.as_ptr()) };
    }
}

#[cfg(feature = "link-vsscript")]
impl Default for Script {
    fn default() -> Self {
        Self::new(None, VssApi::default())
    }
}

// MARK: ScriptError

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[error("VSScript error: {0}")]
pub struct ScriptError(String);

impl ScriptError {
    unsafe fn from_vss(vss: &Script) -> Self {
        unsafe {
            let error = (vss.vssapi.getError)(vss.handle.as_ptr());
            Self(if error.is_null() {
                "Script operation failed without an error message".into()
            } else {
                CStr::from_ptr(error).to_string_lossy().into_owned()
            })
        }
    }
}

#[cfg(all(test, feature = "link-vsscript", feature = "vsscript-44"))]
mod tests {
    use super::*;
    use crate::map::KeyStr;
    #[cfg(feature = "vs-43")]
    use crate::node::Node;

    #[test]
    fn api_44_accessors() -> testresult::TestResult {
        let script = Script::default();
        assert_eq!(script.get_available_outputs_ex(0)?, Vec::<c_int>::new());
        script.evaluate(c"import vapoursynth as vs\nclip = vs.core.std.BlankClip(width=16, height=16, length=1, format=vs.RGB24)\nalpha = vs.core.std.BlankClip(clip, format=vs.GRAY8)\nclip.set_output(0, alpha=alpha)\nclip.set_output(2)\nvalue = 42\n", c"api44.vpy")?;
        let mut variables = script.core()?.create_map();
        for flags in [0, ffi::VSScriptGetFlags::AllowGPUResident as c_int] {
            assert_eq!(script.get_available_outputs_ex(flags)?, vec![0, 2]);
            let output = script.get_output_ex(0, flags)?;
            assert_eq!(output.as_video().unwrap().info().width, 16);
            let alpha = script.get_output_alpha_ex(0, flags)?;
            assert_eq!(alpha.info().width, 16);
            #[cfg(feature = "vs-43")]
            {
                assert_eq!(
                    output.as_video().unwrap().residency(),
                    ffi::VSNodeResidency::CPU
                );
                assert_eq!(alpha.residency(), ffi::VSNodeResidency::CPU);
            }
            script.get_variable_ex(c"value", &mut variables, flags)?;
            assert_eq!(variables.get_int(KeyStr::from_cstr(c"value"), 0)?, 42);
            script.get_variable_ex(c"clip", &mut variables, flags)?;
            assert!(script.get_output_ex(99, flags).is_err());
            assert!(script.get_output_alpha_ex(2, flags).is_err());
            assert!(
                script
                    .get_variable_ex(c"missing", &mut variables, flags)
                    .is_err()
            );
        }
        assert!(script.get_output(99).is_err());
        Ok(())
    }

    #[cfg(feature = "vulkan")]
    #[test]
    fn api_44_gpu_residency() -> testresult::TestResult {
        use crate::frame::Frame;

        let script = Script::default();
        let api = script.get_api();
        let mut info = std::mem::MaybeUninit::<ffi::VSVulkanCoreInfo>::uninit();
        let mut error = [0; 1024];
        let status = unsafe {
            (api.vulkan().getVulkanCoreInfo)(
                script.core()?.as_ptr(),
                info.as_mut_ptr(),
                error.as_mut_ptr(),
                1024,
            )
        };
        if status != 0 {
            let error = unsafe { CStr::from_ptr(error.as_ptr()) }.to_string_lossy();
            if std::env::var_os("VAPOURSYNTH_REQUIRE_VULKAN").is_some() {
                return Err(format!("Vulkan initialization failed: {error}").into());
            }
            eprintln!("Skipping GPU residency test: {error}");
            return Ok(());
        }
        script.evaluate(c"import vapoursynth as vs\nclip = vs.core.std.BlankClip(width=16, height=16, length=1, format=vs.GRAY8)\ngpu = vs.core.std.GPUUpload(clip)\ngpu.set_output(0)\ngpu_frame = gpu.get_frame(0)\n", c"api44_gpu.vpy")?;
        let allow_gpu = ffi::VSScriptGetFlags::AllowGPUResident as c_int;
        for (flags, residency) in [
            (0, ffi::VSNodeResidency::CPU),
            (allow_gpu, ffi::VSNodeResidency::GPU),
        ] {
            let output = script.get_output_ex(0, flags)?;
            let node = output.as_video().unwrap();
            assert_eq!(node.residency(), residency);
            assert_eq!(
                node.get_frame(0)
                    .map_err(|error| error.to_string_lossy().into_owned())?
                    .residency(),
                residency
            );
        }
        let mut variables = script.core()?.create_map();
        assert!(
            script
                .get_variable_ex(c"gpu_frame", &mut variables, 0)
                .is_err()
        );
        variables.clear();
        script.get_variable_ex(c"gpu_frame", &mut variables, allow_gpu)?;
        assert_eq!(
            variables
                .get_video_frame(KeyStr::from_cstr(c"gpu_frame"), 0)?
                .residency(),
            ffi::VSNodeResidency::GPU
        );
        Ok(())
    }
}

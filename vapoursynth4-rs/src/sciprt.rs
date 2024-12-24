use std::{
    ffi::{c_int, CStr},
    ptr::{null_mut, NonNull},
};

use thiserror::Error;
use vapoursynth4_sys::VSNode;

use crate::{
    api::{Api, VssApi},
    core::{Core, CoreRef},
};

use super::ffi;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Script {
    handle: NonNull<ffi::VSScript>,
    vssapi: VssApi,
    api: Api,
}

impl Script {
    /// Creates a new script instance.
    ///
    /// # Panics
    ///
    /// Panics if the script creation fails.
    pub fn new(core: Option<&Core>, vssapi: VssApi, api: Api) -> Self {
        unsafe {
            let handle = NonNull::new((vssapi.createScript)(core.map_or(null_mut(), Core::as_ptr)))
                .expect("Failed to create script");
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
    pub fn core(&self) -> Result<CoreRef, ScriptError> {
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
    pub fn get_output(&self, index: c_int) -> Result<*mut VSNode, ScriptError> {
        unsafe { self.get_ptr_error((self.vssapi.getOutputNode)(self.handle.as_ptr(), index)) }
    }
}

// MARK: Helper
impl Script {
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

#[cfg(feature = "link-library")]
impl Default for Script {
    fn default() -> Self {
        Self::new(None, VssApi::default(), Api::default())
    }
}

// MARK: ScriptError

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[error("VSScript error: {0}")]
pub struct ScriptError(String);

impl ScriptError {
    unsafe fn from_vss(vss: &Script) -> Self {
        unsafe {
            Self(
                CStr::from_ptr((vss.vssapi.getError)(vss.handle.as_ptr()))
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }
}

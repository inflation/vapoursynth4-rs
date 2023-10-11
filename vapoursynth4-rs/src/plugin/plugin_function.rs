use std::{ffi::CStr, ptr::NonNull};

use crate::{api::api, ffi};

use super::Plugin;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct PluginFunction {
    pub(crate) handle: NonNull<ffi::VSPluginFunction>,
}

impl PluginFunction {
    #[must_use]
    pub fn new(handle: NonNull<ffi::VSPluginFunction>) -> Self {
        Self { handle }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const ffi::VSPluginFunction {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn name(&self) -> &CStr {
        unsafe {
            let ptr = (api().getPluginFunctionName)(self.as_ptr().cast_mut());
            CStr::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn arguments(&self) -> &CStr {
        unsafe {
            let ptr = (api().getPluginFunctionArguments)(self.as_ptr().cast_mut());
            CStr::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn return_type(&self) -> &CStr {
        unsafe {
            let ptr = (api().getPluginFunctionReturnType)(self.as_ptr().cast_mut());
            CStr::from_ptr(ptr)
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Functions<'p> {
    cursor: *mut ffi::VSPluginFunction,
    plugin: &'p Plugin,
}

impl<'p> Functions<'p> {
    pub(crate) fn new(plugin: &'p Plugin) -> Functions<'p> {
        Self {
            cursor: std::ptr::null_mut(),
            plugin,
        }
    }
}

impl Iterator for Functions<'_> {
    type Item = PluginFunction;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let ptr = (api().getNextPluginFunction)(self.cursor, self.plugin.as_ptr().cast_mut());
            NonNull::new(ptr).map(PluginFunction::new)
        }
    }
}

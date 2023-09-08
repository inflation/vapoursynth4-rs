use std::{ffi::CStr, ptr::NonNull};

use crate::{
    api,
    core::Core,
    ffi,
    map::{Map, MapRef},
};

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Plugin {
    handle: NonNull<ffi::VSPlugin>,
}

impl Plugin {
    #[must_use]
    pub fn new(handle: NonNull<ffi::VSPlugin>) -> Self {
        Self { handle }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const ffi::VSPlugin {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn name(&self) -> &CStr {
        unsafe {
            let ptr = (api().getPluginName)(self.as_ptr().cast_mut());
            CStr::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn id(&self) -> &CStr {
        unsafe {
            let ptr = (api().getPluginID)(self.as_ptr().cast_mut());
            CStr::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn namespace(&self) -> &CStr {
        unsafe {
            let ptr = (api().getPluginNamespace)(self.as_ptr().cast_mut());
            CStr::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn invoke(&self, name: &CStr, args: MapRef<'_>) -> Map {
        unsafe {
            let ptr = (api().invoke)(self.as_ptr().cast_mut(), name.as_ptr(), args.as_ptr());
            Map::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn functions(&self) -> Functions<'_> {
        Functions::new(self)
    }

    #[must_use]
    pub fn get_function_by_name(&self, name: &CStr) -> Option<PluginFunction> {
        unsafe {
            NonNull::new((api().getPluginFunctionByName)(
                name.as_ptr(),
                self.as_ptr().cast_mut(),
            ))
        }
        .map(|handle| PluginFunction { handle })
    }

    #[must_use]
    pub fn path(&self) -> &CStr {
        unsafe {
            let ptr = (api().getPluginPath)(self.as_ptr().cast_mut());
            CStr::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn version(&self) -> i32 {
        unsafe { (api().getPluginVersion)(self.as_ptr().cast_mut()) }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Plugins<'c> {
    cursor: *mut ffi::VSPlugin,
    core: &'c Core,
}

impl<'c> Plugins<'c> {
    pub(crate) fn new(core: &'c Core) -> Plugins<'c> {
        Self {
            cursor: std::ptr::null_mut(),
            core,
        }
    }
}

impl Iterator for Plugins<'_> {
    type Item = Plugin;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let ptr = (api().getNextPlugin)(self.cursor, self.core.as_ptr().cast_mut());
            NonNull::new(ptr).map(Plugin::new)
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct PluginFunction {
    handle: NonNull<ffi::VSPluginFunction>,
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

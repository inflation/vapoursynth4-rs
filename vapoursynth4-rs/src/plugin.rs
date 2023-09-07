use std::{ffi::CStr, ptr::NonNull};

use crate::{api, ffi, Core, Map, MapRef};

#[macro_export]
macro_rules! declare_plugin {
    ($ident:expr, $namespace:expr, $name:expr, $ver:expr, $api_ver:expr, $flags:expr $(,)*) => {
        #[no_mangle]
        pub unsafe extern "system" fn VapourSynthPluginInit2(
            plugin: *mut vapoursynth4_sys::VSPlugin,
            vspapi: *const vapoursynth4_sys::VSPLUGINAPI,
        ) {
            ((*vspapi).configPlugin)(
                std::ffi::CString::new($ident).unwrap().as_ptr(),
                std::ffi::CString::new($namespace).unwrap().as_ptr(),
                std::ffi::CString::new($name).unwrap().as_ptr(),
                $ver,
                $api_ver,
                $flags,
                plugin,
            );

            ((*vspapi).registerFunction)(
                std::ffi::CString::new("Filter").unwrap().as_ptr(),
                std::ffi::CString::new("clip:vnode;").unwrap().as_ptr(),
                CStr::from_bytes_with_nul_unchecked(b"clip:vnode;\0").as_ptr(),
                filter_create,
                null_mut(),
                plugin,
            );
        }
    };
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Plugin {
    handle: NonNull<ffi::VSPlugin>,
}

impl Plugin {
    #[must_use]
    pub fn new(handle: NonNull<ffi::VSPlugin>) -> Self {
        Self { handle }
    }

    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSPlugin) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr),
        }
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

pub mod plugin_function;
pub mod types;

use std::{borrow::Borrow, ffi::CStr, ptr::NonNull};

use crate::{api::Api, core::Core, ffi, map::Map};

pub use plugin_function::*;
pub use types::*;

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Plugin {
    handle: NonNull<ffi::VSPlugin>,
    api: Api,
}

unsafe impl Send for Plugin {}
unsafe impl Sync for Plugin {}

impl Plugin {
    #[must_use]
    pub fn new(handle: NonNull<ffi::VSPlugin>, api: Api) -> Self {
        Self { handle, api }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const ffi::VSPlugin {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn name(&self) -> &CStr {
        unsafe {
            let ptr = (self.api.getPluginName)(self.as_ptr().cast_mut());
            CStr::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn id(&self) -> &CStr {
        unsafe {
            let ptr = (self.api.getPluginID)(self.as_ptr().cast_mut());
            CStr::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn namespace(&self) -> &CStr {
        unsafe {
            let ptr = (self.api.getPluginNamespace)(self.as_ptr().cast_mut());
            CStr::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn invoke(&self, name: &CStr, args: impl Borrow<Map>) -> Map {
        unsafe {
            let ptr = (self.api.invoke)(
                self.as_ptr().cast_mut(),
                name.as_ptr(),
                args.borrow().as_ptr(),
            );
            Map::from_ptr(ptr, self.api)
        }
    }

    #[must_use]
    pub fn functions(&self) -> Functions<'_> {
        Functions::new(self)
    }

    #[must_use]
    pub fn get_function_by_name(&self, name: &CStr) -> Option<PluginFunction> {
        unsafe {
            NonNull::new((self.api.getPluginFunctionByName)(
                name.as_ptr(),
                self.as_ptr().cast_mut(),
            ))
        }
        .map(|handle| PluginFunction::from_ptr(handle, self.api))
    }

    #[must_use]
    pub fn path(&self) -> &CStr {
        unsafe {
            let ptr = (self.api.getPluginPath)(self.as_ptr().cast_mut());
            CStr::from_ptr(ptr)
        }
    }

    #[must_use]
    pub fn version(&self) -> i32 {
        unsafe { (self.api.getPluginVersion)(self.as_ptr().cast_mut()) }
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
            let api = self.core.api();
            let ptr = (api.getNextPlugin)(self.cursor, self.core.as_ptr());
            NonNull::new(ptr).map(|p| {
                self.cursor = ptr;
                Plugin::new(p, api)
            })
        }
    }
}

#[macro_export]
macro_rules! declare_plugin {
    ($id:literal, $name:literal, $desc:literal,
        $version:expr,
        $api_version:expr, $flags:expr
        $(, ($filter:tt, $data:expr) )*
    ) => {
        #[unsafe(no_mangle)]
        pub unsafe extern "system-unwind" fn VapourSynthPluginInit2(
            plugin: *mut $crate::ffi::VSPlugin,
            vspapi: *const $crate::ffi::VSPLUGINAPI,
        ) {
            unsafe {
                ((*vspapi).configPlugin)(
                    $id.as_ptr(),
                    $name.as_ptr(),
                    $desc.as_ptr(),
                    $crate::utils::make_version($version.0, $version.1),
                    $crate::VAPOURSYNTH_API_VERSION,
                    $flags,
                    plugin,
                );

                $(
                    $crate::node::FilterRegister::<$filter>::new($data).register(plugin, vspapi);
                )*
            }
        }
    };
}

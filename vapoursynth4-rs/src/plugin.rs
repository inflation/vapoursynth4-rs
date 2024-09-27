pub mod plugin_function;
pub mod types;

use std::{ffi::CStr, ptr::NonNull};

use crate::{
    api::api,
    core::Core,
    ffi,
    map::{Map, MapRef},
};

pub use plugin_function::*;
pub use types::*;

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
    pub fn invoke(&self, name: &CStr, args: &MapRef) -> Map {
        debug_assert!(!args.as_ptr().is_null());
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

#[macro_export]
macro_rules! declare_plugin {
    ($id:literal, $name:literal, $desc:literal,
        $version:expr,
        $api_version:expr, $flags:expr
        $(, ($filter:tt, $data:expr) )*
    ) => {
        #[no_mangle]
        pub unsafe extern "system" fn VapourSynthPluginInit2(
            plugin: *mut $crate::ffi::VSPlugin,
            vspapi: *const $crate::ffi::VSPLUGINAPI,
        ) {
            ((*vspapi).configPlugin)(
                cstr!($id).as_ptr(),
                cstr!($name).as_ptr(),
                cstr!($desc).as_ptr(),
                $crate::utils::make_version($version.0, $version.1),
                $crate::VAPOURSYNTH_API_VERSION,
                $flags,
                plugin,
            );

            $(
                $crate::node::FilterRegister::<$filter>::new($data).register(plugin, vspapi);
            )*
        }
    };
}

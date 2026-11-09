pub mod plugin_function;
pub mod types;

use std::{borrow::Borrow, ffi::CStr, ptr::NonNull};

use crate::{api::Api, core::CoreHandle, ffi, map::Map};

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
    core: &'c CoreHandle,
}

impl<'c> Plugins<'c> {
    pub(crate) fn new(core: &'c CoreHandle) -> Plugins<'c> {
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

#[cfg(all(test, feature = "link-vs"))]
mod tests {
    use crate::{
        core::Core,
        key,
        map::{AppendMode, Value},
    };

    #[test]
    fn discovers_and_invokes_builtin_plugin() -> testresult::TestResult {
        let core = Core::builder()
            .disable_auto_loading()
            .enable_graph_inspection()
            .build();
        let plugin = core.get_plugin_by_namespace(c"std").unwrap();
        assert_eq!(plugin.namespace(), c"std");
        assert_eq!(plugin.id(), c"com.vapoursynth.std");
        assert!(!plugin.name().is_empty());
        assert!(plugin.version() > 0);
        assert_eq!(core.get_plugin_by_id(plugin.id()), Some(plugin));
        assert!(core.get_plugin_by_namespace(c"missing").is_none());
        assert!(core.get_plugin_by_id(c"missing").is_none());

        let plugin = core
            .plugins()
            .find(|plugin| plugin.namespace() == c"std")
            .unwrap();
        let blank = plugin.get_function_by_name(c"BlankClip").unwrap();
        assert_eq!(blank.name(), c"BlankClip");
        assert!(blank.arguments().to_bytes().starts_with(b"clip:vnode"));
        assert!(blank.return_type().to_bytes().starts_with(b"clip:vnode"));
        assert!(plugin.functions().any(|function| function == blank));
        assert!(plugin.get_function_by_name(c"Missing").is_none());

        let mut args = core.create_map();
        args.set(key!(c"width"), Value::Int(16), AppendMode::Replace)?;
        args.set(key!(c"height"), Value::Int(16), AppendMode::Replace)?;
        let output = plugin.invoke(blank.name(), &args);
        assert!(output.get_error().is_none());
        let clip = output.get_video_node(key!(c"clip"), 0)?;
        assert_eq!(clip.info().width, 16);
        #[cfg(feature = "vs-graph")]
        {
            use crate::node::Node;
            assert_eq!(clip.creation_function_name(0), Some(c"BlankClip"));
            assert_eq!(clip.creation_plugin_id(0), Some(c"com.vapoursynth.std"));
            assert_eq!(clip.creation_plugin_namespace(0), Some(c"std"));
            let arguments = clip.creation_function_arguments(0).unwrap();
            assert_eq!(arguments.get_int(key!(c"width"), 0)?, 16);
            assert!(clip.creation_function_name(20).is_none());
            assert!(clip.creation_function_arguments(20).is_none());
        }
        assert!(plugin.invoke(c"Missing", &args).get_error().is_some());
        Ok(())
    }
}

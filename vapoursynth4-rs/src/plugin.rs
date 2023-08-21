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

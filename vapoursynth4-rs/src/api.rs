use std::{ops::Deref, ptr::NonNull};

use vapoursynth4_sys as ffi;

#[derive(Clone, Copy)]
pub struct ApiRef {
    handle: NonNull<ffi::VSAPI>,
}

impl ApiRef {
    #[must_use]
    pub fn new() -> Option<Self> {
        Self::new_with_version(ffi::VAPOURSYNTH_API_VERSION)
    }

    #[must_use]
    pub fn new_with(major: u16, minor: u16) -> Option<Self> {
        Self::new_with_version(ffi::VS_MAKE_VERSION(major, minor))
    }

    #[must_use]
    pub fn new_with_version(version: i32) -> Option<Self> {
        let handle = NonNull::new(unsafe { ffi::getVapourSynthAPI(version) }.cast_mut())?;
        Some(Self { handle })
    }

    #[must_use]
    pub fn get_version(&self) -> i32 {
        unsafe { (self.getAPIVersion)() }
    }
}

impl Deref for ApiRef {
    type Target = ffi::VSAPI;

    fn deref(&self) -> &Self::Target {
        unsafe { self.handle.as_ref() }
    }
}

#[cfg(test)]
mod tests {
    use crate::Core;

    use super::*;

    #[test]
    fn test_api() {
        let api = ApiRef::new().unwrap();
        let core = Core::new(api);
        let info = core.get_info();
        println!("{info:?}");
    }
}

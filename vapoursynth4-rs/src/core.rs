use std::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use ffi::VSCoreInfo;
use vapoursynth4_sys as ffi;

use crate::ApiRef;

pub struct Core<'c> {
    handle: NonNull<ffi::VSCore>,
    _marker: PhantomData<&'c ()>,
    api: ApiRef,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct CoreBuilder {
    flags: i32,
    max_cache_size: Option<i64>,
    thread_count: Option<i32>,
}

impl CoreBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn build<'c>(self, api: ApiRef) -> Core<'c> {
        let mut core = Core::new_with(api, self.flags);
        if let Some(size) = self.max_cache_size {
            core.set_max_cache_size(size);
        }
        if let Some(count) = self.thread_count {
            core.set_thread_count(count);
        }
        core
    }

    pub fn enable_graph_inspection(&mut self) -> &mut Self {
        self.flags |= ffi::VSCoreCreationFlags::ccfEnableGraphInspection as i32;
        self
    }

    pub fn disable_auto_loading(&mut self) -> &mut Self {
        self.flags |= ffi::VSCoreCreationFlags::ccfDisableAutoLoading as i32;
        self
    }

    pub fn disable_library_unloading(&mut self) -> &mut Self {
        self.flags |= ffi::VSCoreCreationFlags::ccfDisableLibraryUnloading as i32;
        self
    }

    pub fn max_cache_size(&mut self, size: i64) -> &mut Self {
        self.max_cache_size = Some(size);
        self
    }

    pub fn thread_count(&mut self, count: i32) -> &mut Self {
        self.thread_count = Some(count);
        self
    }
}

impl<'c> Core<'c> {
    #[must_use]
    pub fn new(api: ApiRef) -> Self {
        Self::new_with(api, 0)
    }

    #[must_use]
    pub fn new_with(api: ApiRef, flags: i32) -> Self {
        let core = unsafe { (api.createCore)(flags) };
        unsafe {
            Self {
                // Safety: `core` is always a valid pointer to a `VSCore` instance.
                handle: NonNull::new_unchecked(core),
                _marker: PhantomData,
                api,
            }
        }
    }

    #[must_use]
    pub fn api(&self) -> ApiRef {
        self.api
    }

    pub fn set_max_cache_size(&mut self, size: i64) {
        unsafe {
            (self.api.setMaxCacheSize)(size, self.handle.as_ptr());
        }
    }

    pub fn set_thread_count(&mut self, count: i32) {
        unsafe {
            (self.api.setThreadCount)(count, self.handle.as_ptr());
        }
    }

    #[must_use]
    pub fn get_info(&self) -> VSCoreInfo {
        unsafe {
            let mut info = MaybeUninit::uninit();
            (self.api.getCoreInfo)(self.handle.as_ptr(), info.as_mut_ptr());
            info.assume_init()
        }
    }
}

impl Drop for Core<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.api.freeCore)(self.handle.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let api = ApiRef::new().unwrap();
        let core = CoreBuilder::new()
            .enable_graph_inspection()
            .disable_auto_loading()
            .disable_library_unloading()
            .max_cache_size(1024)
            .thread_count(4)
            .build(api);
        assert_eq!(core.get_info().maxFramebufferSize, 1024);
        assert_eq!(core.get_info().numThreads, 4);
    }
}

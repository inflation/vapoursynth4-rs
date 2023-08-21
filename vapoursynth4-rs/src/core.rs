/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{mem::MaybeUninit, ptr::NonNull};

use ffi::VSCoreInfo;
use vapoursynth4_sys as ffi;

use crate::{api, ApiRef, API};

pub struct Core {
    handle: NonNull<ffi::VSCore>,
}

impl Core {
    #[must_use]
    pub fn new() -> Self {
        Self::new_with(ApiRef::default(), 0)
    }

    fn new_with(api: ApiRef, flags: i32) -> Self {
        API.set(api);
        let core = unsafe { (api.createCore)(flags) };
        unsafe {
            Self {
                // Safety: `core` is always a valid pointer to a `VSCore` instance.
                handle: NonNull::new_unchecked(core),
            }
        }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *mut ffi::VSCore {
        self.handle.as_ptr()
    }

    pub fn set_max_cache_size(&mut self, size: i64) {
        unsafe {
            (api().setMaxCacheSize)(size, self.handle.as_ptr());
        }
    }

    pub fn set_thread_count(&mut self, count: i32) {
        unsafe {
            (api().setThreadCount)(count, self.handle.as_ptr());
        }
    }

    #[must_use]
    pub fn get_info(&self) -> VSCoreInfo {
        unsafe {
            let mut info = MaybeUninit::uninit();
            (api().getCoreInfo)(self.handle.as_ptr(), info.as_mut_ptr());
            info.assume_init()
        }
    }
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe {
            (api().freeCore)(self.handle.as_ptr());
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct CoreBuilder {
    flags: i32,
    api: Option<ApiRef>,
    max_cache_size: Option<i64>,
    thread_count: Option<i32>,
}

impl CoreBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn build(self) -> Core {
        let mut core = Core::new_with(self.api.unwrap_or_default(), self.flags);
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

    pub fn api(&mut self, api: ApiRef) -> &mut Self {
        self.api = Some(api);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api() {
        let core = Core::new();
        let info = core.get_info();
        println!("{info:?}");
    }

    #[test]
    fn test_builder() {
        let core = CoreBuilder::new()
            .enable_graph_inspection()
            .disable_auto_loading()
            .disable_library_unloading()
            .max_cache_size(1024)
            .thread_count(4)
            .build();
        assert_eq!(core.get_info().maxFramebufferSize, 1024);
        assert_eq!(core.get_info().numThreads, 4);
    }
}

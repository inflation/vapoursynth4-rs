/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::ptr::NonNull;

use vapoursynth4_sys as ffi;

pub struct FrameRef {
    handle: NonNull<ffi::VSFrame>,
}

impl FrameRef {
    pub(crate) unsafe fn from_raw(ptr: *const ffi::VSFrame) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
        }
    }

    #[must_use]
    pub fn as_mut_ptr(&self) -> *mut ffi::VSFrame {
        self.handle.as_ptr()
    }
}

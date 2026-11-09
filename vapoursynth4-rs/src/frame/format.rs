use std::ffi::{CStr, c_char};

use crate::ffi;

pub type VideoFormat = ffi::VSVideoFormat;
pub type AudioFormat = ffi::VSAudioFormat;

pub(crate) struct FormatName {
    buffer: [u8; 32],
}

impl FormatName {
    pub fn new() -> Self {
        Self { buffer: [0; 32] }
    }

    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        CStr::from_bytes_until_nul(&self.buffer).ok()?.to_str().ok()
    }

    pub fn as_mut_ptr(&mut self) -> *mut c_char {
        self.buffer.as_mut_ptr().cast()
    }
}

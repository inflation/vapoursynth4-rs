use std::{
    ffi::{c_char, CStr},
    fmt::Display,
    str,
};

use crate::ffi;

pub type VideoFormat = ffi::VSVideoFormat;
pub type AudioFormat = ffi::VSAudioFormat;

pub(crate) struct FormatName {
    pub buffer: [u8; 32],
}

impl FormatName {
    pub fn new() -> Self {
        Self { buffer: [0; 32] }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        // SAFETY: The buffer is guaranteed to be alphanumeric or underscore and null-terminated.
        unsafe {
            str::from_utf8_unchecked(CStr::from_bytes_with_nul_unchecked(&self.buffer).to_bytes())
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut c_char {
        self.buffer.as_mut_ptr().cast()
    }
}

impl Display for FormatName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

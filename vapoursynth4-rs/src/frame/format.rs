use std::{ffi::CStr, fmt::Display};

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
}

impl Display for FormatName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cstr = unsafe { CStr::from_bytes_until_nul(&self.buffer).unwrap_unchecked() };
        cstr.to_str()
            .map_err(|_| std::fmt::Error)
            .and_then(|s| f.write_str(s))
    }
}

use std::{ffi::CStr, fmt::Display};

use crate::{api, ffi};

pub type VideoFormat = ffi::VSVideoFormat;
pub type AudioFormat = ffi::VSAudioFormat;

struct FormatName {
    buffer: [u8; 32],
}

impl FormatName {
    fn new() -> Self {
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

pub trait FormatExt {
    fn name(&self) -> Option<String>;
}

impl FormatExt for VideoFormat {
    fn name(&self) -> Option<String> {
        let mut buffer = FormatName::new();
        if 0 == unsafe { (api().getVideoFormatName)(self, buffer.buffer.as_mut_ptr().cast()) } {
            None
        } else {
            Some(buffer.to_string())
        }
    }
}

impl FormatExt for AudioFormat {
    fn name(&self) -> Option<String> {
        let mut buffer = FormatName::new();
        if 0 == unsafe { (api().getAudioFormatName)(self, buffer.buffer.as_mut_ptr().cast()) } {
            None
        } else {
            Some(buffer.to_string())
        }
    }
}

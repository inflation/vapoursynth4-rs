/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{
    ffi::{c_char, CStr},
    fmt::Display,
    str,
};

use crate::{api::Api, ffi};

pub type VideoFormat = ffi::VSVideoFormat;
pub type AudioFormat = ffi::VSAudioFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct FormatName {
    buffer: [u8; 32],
}

impl FormatName {
    #[must_use]
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

pub trait GetFormatName {
    fn get_format_name(&self, api: Api) -> FormatName;
}

impl GetFormatName for VideoFormat {
    fn get_format_name(&self, api: Api) -> FormatName {
        let mut buffer = FormatName::default();
        unsafe {
            (api.getVideoFormatName)(self, buffer.as_mut_ptr());
        }
        buffer
    }
}

impl GetFormatName for AudioFormat {
    fn get_format_name(&self, api: Api) -> FormatName {
        let mut buffer = FormatName::default();
        unsafe {
            (api.getAudioFormatName)(self, buffer.as_mut_ptr());
        }
        buffer
    }
}

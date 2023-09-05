use std::mem::MaybeUninit;

use crate::{api, ffi, ColorFamily, Core, SampleType};

pub type VideoFormat = ffi::VSVideoFormat;
pub type AudioFormat = ffi::VSAudioFormat;

pub enum Format<'f> {
    Video(&'f VideoFormat),
    Audio(&'f AudioFormat),
}

struct FormatName {
    buffer: [u8; 32],
}

impl FormatName {
    fn new() -> Self {
        Self { buffer: [0; 32] }
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
            Some(unsafe { String::from_utf8_unchecked(buffer.buffer.to_vec()) })
        }
    }
}

impl FormatExt for AudioFormat {
    fn name(&self) -> Option<String> {
        let mut buffer = FormatName::new();
        if 0 == unsafe { (api().getAudioFormatName)(self, buffer.buffer.as_mut_ptr().cast()) } {
            None
        } else {
            Some(unsafe { String::from_utf8_unchecked(buffer.buffer.to_vec()) })
        }
    }
}

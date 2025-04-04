/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

//! VSHelper4.h

#![allow(clippy::cast_possible_truncation)]

use std::ffi::{c_int, c_void};

use crate::{
    VSAPI, VSAudioFormat, VSAudioInfo, VSColorFamily, VSCore, VSPresetVideoFormat, VSVideoFormat,
    VSVideoInfo,
};

/// Convenience function for checking if the format never changes between frames
#[inline]
#[must_use]
pub const fn is_constant_video_format(vi: &VSVideoInfo) -> bool {
    vi.height > 0
        && vi.width > 0
        && vi.format.color_family as i32 != VSColorFamily::Undefined as i32
}

/// Convenience function to check if two clips have the same format
/// (unknown/changeable will be considered the same too)
#[inline]
#[must_use]
pub const fn is_same_video_format(v1: &VSVideoFormat, v2: &VSVideoFormat) -> bool {
    v1.color_family as i32 == v2.color_family as i32
        && v1.sample_type as i32 == v2.sample_type as i32
        && v1.bits_per_sample == v2.bits_per_sample
        && v1.sub_sampling_w == v2.sub_sampling_w
        && v1.sub_sampling_h == v2.sub_sampling_h
}

impl VSAPI {
    /// Convenience function to check if a clip has the same format as a format id
    ///
    /// # Safety
    ///
    /// `core` must be valid
    #[inline]
    pub unsafe fn is_same_video_preset_format(
        &self,
        preset_format: VSPresetVideoFormat,
        v: &VSVideoFormat,
        core: *mut VSCore,
    ) -> bool {
        unsafe {
            (self.queryVideoFormatID)(
                v.color_family,
                v.sample_type,
                v.bits_per_sample,
                v.sub_sampling_w,
                v.sub_sampling_h,
                core,
            ) == preset_format as u32
        }
    }
}

/// Convenience function to check for if two clips have the same format
/// (but not framerate) while also including width and height
/// (unknown/changeable will be considered the same too)
#[inline]
#[must_use]
pub const fn is_same_video_info(v1: &VSVideoInfo, v2: &VSVideoInfo) -> bool {
    v1.height == v2.height && v1.width == v2.width && is_same_video_format(&v1.format, &v2.format)
}

/// Convenience function to check for if two clips have the same format while also including
/// `sampleRate` (unknown/changeable will be considered the same too)
#[inline]
#[must_use]
pub const fn is_same_audio_format(a1: &VSAudioFormat, a2: &VSAudioFormat) -> bool {
    a1.bits_per_sample == a2.bits_per_sample
        && a1.sample_type as i32 == a2.sample_type as i32
        && a1.channel_layout == a2.channel_layout
}

/// Convenience function to check for if two clips have the same format while also including
/// `sampleRate` (unknown/changeable will be considered the same too)
#[inline]
#[must_use]
pub const fn is_same_audio_info(a1: &VSAudioInfo, a2: &VSAudioInfo) -> bool {
    a1.sample_rate == a2.sample_rate && is_same_audio_format(&a1.format, &a2.format)
}

/// Multiplies and divides a rational number,
/// such as a frame duration, in place and reduces the result
// TODO: use `const` when available: https://github.com/rust-lang/rust/issues/57349
#[inline]
pub fn muldiv_rational(num: &mut i64, den: &mut i64, mul: i64, div: i64) {
    // do nothing if the rational number is invalid
    if *den == 0 {
        return;
    }

    *num *= mul;
    *den *= div;
    let mut a = *num;
    let mut b = *den;
    while b != 0 {
        let t = a;
        a = b;
        b = t % b;
    }
    if a < 0 {
        a = -a;
    }

    *num /= a;
    *den /= a;
}

/// Reduces a rational number
#[inline]
pub fn reduce_rational(num: &mut i64, den: &mut i64) {
    muldiv_rational(num, den, 1, 1);
}

/// Add two rational numbers and reduces the result
#[inline]
pub fn add_rational(num: &mut i64, den: &mut i64, mut addnum: i64, addden: i64) {
    // Do nothing if the rational number is invalid
    if *den == 0 {
        return;
    }

    if *den == addden {
        *num += addnum;
    } else {
        let temp = addden;
        addnum *= *den;
        // addden *= *den;
        *num *= temp;
        *den *= temp;

        *num += addnum;

        reduce_rational(num, den);
    }
}

/// Converts an int64 to int with saturation, useful to silence warnings when reading
/// int properties among other things
#[inline]
#[must_use]
pub const fn int64_to_int_s(i: i64) -> c_int {
    if i > c_int::MAX as i64 {
        c_int::MAX
    } else if i < c_int::MIN as i64 {
        c_int::MIN
    } else {
        i as c_int
    }
}

/// Converts a double to float with saturation, useful to silence warnings when reading
/// float properties among other things
#[inline]
#[must_use]
pub const fn double_to_float_s(d: f64) -> f32 {
    d as f32
}

/// Copies bytes from one plane to another. Basically, it is memcpy in a loop.
///
/// # Safety
/// `srcp` and `dstp` must be valid and not overlapping
#[inline]
pub unsafe fn bitblt(
    dstp: *mut c_void,
    dst_stride: isize,
    srcp: *const c_void,
    src_stride: isize,
    row_size: usize,
    height: usize,
) {
    if height != 0 {
        if src_stride == dst_stride && src_stride == row_size as isize {
            unsafe { dstp.copy_from_nonoverlapping(srcp, row_size * height) };
        } else {
            let mut srcp8 = srcp.cast::<u8>();
            let mut dstp8 = dstp.cast::<u8>();
            let mut i = 0;
            while i < height {
                unsafe { dstp8.copy_from_nonoverlapping(srcp8, row_size) };
                srcp8 = srcp8.wrapping_offset(src_stride);
                dstp8 = dstp8.wrapping_offset(dst_stride);
                i += 1;
            }
        }
    }
}

// Check if the frame dimensions are valid for a given format
// returns non-zero for valid width and height
#[inline]
#[must_use]
pub const fn are_valid_dimensions(fi: &VSVideoFormat, width: c_int, height: c_int) -> bool {
    width % (1 << fi.sub_sampling_w) == 0 && height % (1 << fi.sub_sampling_h) == 0
}

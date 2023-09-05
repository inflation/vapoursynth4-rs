use crate::ffi;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MediaType {
    Video,
    Audio,
}

impl From<ffi::VSMediaType> for MediaType {
    fn from(media_type: ffi::VSMediaType) -> Self {
        match media_type {
            ffi::VSMediaType::mtVideo => Self::Video,
            ffi::VSMediaType::mtAudio => Self::Audio,
        }
    }
}

pub type VideoInfo = ffi::VSVideoInfo;
pub type AudioInfo = ffi::VSAudioInfo;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ColorFamily {
    Undefined,
    Gray,
    Rgb,
    Yuv,
}

impl From<ffi::VSColorFamily> for ColorFamily {
    fn from(color_family: ffi::VSColorFamily) -> Self {
        match color_family {
            ffi::VSColorFamily::cfUndefined => Self::Undefined,
            ffi::VSColorFamily::cfGray => Self::Gray,
            ffi::VSColorFamily::cfRGB => Self::Rgb,
            ffi::VSColorFamily::cfYUV => Self::Yuv,
        }
    }
}

impl From<ColorFamily> for ffi::VSColorFamily {
    fn from(color_family: ColorFamily) -> Self {
        match color_family {
            ColorFamily::Undefined => ffi::VSColorFamily::cfUndefined,
            ColorFamily::Gray => ffi::VSColorFamily::cfGray,
            ColorFamily::Rgb => ffi::VSColorFamily::cfRGB,
            ColorFamily::Yuv => ffi::VSColorFamily::cfYUV,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum SampleType {
    Integer,
    Float,
}

impl From<ffi::VSSampleType> for SampleType {
    fn from(sample_type: ffi::VSSampleType) -> Self {
        match sample_type {
            ffi::VSSampleType::stInteger => Self::Integer,
            ffi::VSSampleType::stFloat => Self::Float,
        }
    }
}

impl From<SampleType> for ffi::VSSampleType {
    fn from(sample_type: SampleType) -> Self {
        match sample_type {
            SampleType::Integer => ffi::VSSampleType::stInteger,
            SampleType::Float => ffi::VSSampleType::stFloat,
        }
    }
}

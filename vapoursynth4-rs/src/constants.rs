use vapoursynth4_sys as ffi;

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

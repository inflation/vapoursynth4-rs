/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

// VapourSynth4.h
//! This is `VapourSynth`'s main header file.
//! Plugins and applications that use the library must include it.
//!
//! `VapourSynth`'s public API is all C.

// #![allow(non_upper_case_globals)]
#![allow(clippy::enum_glob_use)]

use std::ffi::*;

use super::opaque_struct;
use super::*;

/// Major API version.
pub const VAPOURSYNTH_API_MAJOR: u16 = 4;
/// Minor API version. It is bumped when new functions are added to [`VSAPI`]
/// or core behavior is noticeably changed.
pub const VAPOURSYNTH_API_MINOR: u16 = if cfg!(feature = "vs-41") { 1 } else { 0 };
/// API version. The high 16 bits are [`VAPOURSYNTH_API_MAJOR`], the low 16 bits are
/// [`VAPOURSYNTH_API_MINOR`].
pub const VAPOURSYNTH_API_VERSION: i32 =
    vs_make_version(VAPOURSYNTH_API_MAJOR, VAPOURSYNTH_API_MINOR);

/// The number of audio samples in an audio frame. It is a static number to
/// make it possible to calculate which audio frames are needed to retrieve specific samples.
pub const VS_AUDIO_FRAME_SAMPLES: i32 = 3072;

opaque_struct!(
    /// A frame that can hold audio or video data.
    ///
    /// Each row of pixels in a frame is guaranteed to have an alignment of at least 32 bytes.
    /// Two frames with the same width and bytes per sample are guaranteed to have the same stride.
    ///
    /// Audio data is also guaranteed to be at least 32 byte aligned.
    ///
    /// Any data can be attached to a frame, using a VSMap.
    VSFrame,
    /// A reference to a node in the constructed filter graph. Its primary use is as an argument
    /// to other filter or to request frames from.
    VSNode,
    /// The core represents one instance of VapourSynth.
    /// Every core individually loads plugins and keeps track of memory.
    VSCore,
    /// A VapourSynth plugin. There are a few of these built into the core,
    /// and therefore available at all times: the basic filters (identifier `com.vapoursynth.std`,
    /// namespace `std`), the resizers (identifier `com.vapoursynth.resize`, namespace `resize`),
    /// and the Avisynth compatibility module, if running in Windows
    /// (identifier `com.vapoursynth.avisynth`, namespace `avs`).
    ///
    /// The Function Reference describes how to load VapourSynth and Avisynth plugins.
    ///
    /// A [`VSPlugin`] instance is constructed by the core when loading a plugin
    /// (.so / .dylib / .dll), and the pointer is passed to the plugin's
    /// `VapourSynthPluginInit2()` function.
    ///
    /// A VapourSynth plugin can export any number of filters.
    ///
    /// Plugins have a few attributes:
    ///
    /// - An identifier, which must be unique among all VapourSynth plugins in existence,
    ///   because this is what the core uses to make sure a plugin only gets loaded once.
    /// - A namespace, also unique. The filters exported by a plugin end up
    ///     in the plugin's namespace.
    /// - A full name, which is used by the core in a few error messages.
    /// - The version of the plugin.
    /// - The VapourSynth API version the plugin requires.
    /// - A file name.
    ///
    /// Things you can do with a [`VSPlugin`]:
    ///
    /// - Enumerate all the filters it exports, using
    ///   [`getNextPluginFunction()`](VSAPI::getNextPluginFunction).
    /// - Invoke one of its filters, using [`invoke()`](VSAPI::invoke).
    /// - Get its location in the file system, using [`getPluginPath()`](VSAPI::getPluginPath).
    ///
    /// All loaded plugins (including built-in) can be enumerated with
    /// [`getNextPlugin()`](VSAPI::getNextPlugin).
    ///
    /// Once loaded, a plugin only gets unloaded when the VapourSynth core is freed.
    VSPlugin,
    /// A function belonging to a Vapoursynth plugin. This object primarily exists
    /// so a plugin's name, argument list and return type can be queried by editors.
    ///
    /// One peculiarity is that plugin functions cannot be invoked using a
    /// [`VSPluginFunction`] pointer but is instead done using [`invoke()`](VSAPI::invoke)
    /// which takes a [`VSPlugin`] and the function name as a string.
    VSPluginFunction,
    /// Holds a reference to a function that may be called.
    /// This type primarily exists so functions can be shared between
    /// the scripting layer and plugins in the core.
    VSFunction,
    /// [`VSMap`] is a container that stores (key, value) pairs.
    /// The keys are strings and the values can be (arrays of) integers,
    /// floating point numbers, arrays of bytes, [`VSNode`], [`VSFrame`], or [`VSFunction`].
    ///
    /// The pairs in a [`VSMap`] are sorted by key.
    ///
    /// **In VapourSynth, [`VSMap`]s have several uses:**
    /// - storing filters' arguments and return values
    /// - storing user-defined functions' arguments and return values
    /// - storing the properties attached to frames
    ///
    /// Only alphanumeric characters and the underscore may be used in keys.
    ///
    /// Creating and destroying a map can be done with [`createMap()`](VSAPI::createMap) and
    /// [`freeMap()`](VSAPI::freeMap), respectively.
    ///
    /// A map's contents can be retrieved and modified using a number of functions,
    /// all prefixed with "map".
    ///
    /// A map's contents can be erased with [`clearMap()`](VSAPI::clearMap).
    VSMap,
    /// Opaque type representing a registered logger.
    VSLogHandle,
    /// Opaque type representing the current frame request in a filter.
    VSFrameContext
);

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSColorFamily {
    Undefined = 0,
    Gray = 1,
    RGB = 2,
    YUV = 3,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSSampleType {
    Integer = 0,
    Float = 1,
}

const fn vs_make_video_id(
    color_family: VSColorFamily,
    sample_type: VSSampleType,
    bits_per_sample: isize,
    sub_sampling_w: isize,
    sub_sampling_h: isize,
) -> isize {
    ((color_family as isize) << 28)
        | ((sample_type as isize) << 24)
        | (bits_per_sample << 16)
        | (sub_sampling_w << 8)
        | sub_sampling_h
}

use VSColorFamily::*;
use VSSampleType::*;

/// The presets suffixed with H and S have floating point sample type.
/// The H and S suffixes stand for half precision and single precision, respectively.
/// All formats are planar.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSPresetVideoFormat {
    None = 0,

    Gray8 = vs_make_video_id(Gray, Integer, 8, 0, 0),
    Gray9 = vs_make_video_id(Gray, Integer, 9, 0, 0),
    Gray10 = vs_make_video_id(Gray, Integer, 10, 0, 0),
    Gray12 = vs_make_video_id(Gray, Integer, 12, 0, 0),
    Gray14 = vs_make_video_id(Gray, Integer, 14, 0, 0),
    Gray16 = vs_make_video_id(Gray, Integer, 16, 0, 0),
    Gray32 = vs_make_video_id(Gray, Integer, 32, 0, 0),

    GrayH = vs_make_video_id(Gray, Float, 16, 0, 0),
    GrayS = vs_make_video_id(Gray, Float, 32, 0, 0),

    YUV410P8 = vs_make_video_id(YUV, Integer, 8, 2, 2),
    YUV411P8 = vs_make_video_id(YUV, Integer, 8, 2, 0),
    YUV440P8 = vs_make_video_id(YUV, Integer, 8, 0, 1),

    YUV420P8 = vs_make_video_id(YUV, Integer, 8, 1, 1),
    YUV422P8 = vs_make_video_id(YUV, Integer, 8, 1, 0),
    YUV444P8 = vs_make_video_id(YUV, Integer, 8, 0, 0),

    YUV420P9 = vs_make_video_id(YUV, Integer, 9, 1, 1),
    YUV422P9 = vs_make_video_id(YUV, Integer, 9, 1, 0),
    YUV444P9 = vs_make_video_id(YUV, Integer, 9, 0, 0),

    YUV420P10 = vs_make_video_id(YUV, Integer, 10, 1, 1),
    YUV422P10 = vs_make_video_id(YUV, Integer, 10, 1, 0),
    YUV444P10 = vs_make_video_id(YUV, Integer, 10, 0, 0),

    YUV420P12 = vs_make_video_id(YUV, Integer, 12, 1, 1),
    YUV422P12 = vs_make_video_id(YUV, Integer, 12, 1, 0),
    YUV444P12 = vs_make_video_id(YUV, Integer, 12, 0, 0),

    YUV420P14 = vs_make_video_id(YUV, Integer, 14, 1, 1),
    YUV422P14 = vs_make_video_id(YUV, Integer, 14, 1, 0),
    YUV444P14 = vs_make_video_id(YUV, Integer, 14, 0, 0),

    YUV420P16 = vs_make_video_id(YUV, Integer, 16, 1, 1),
    YUV422P16 = vs_make_video_id(YUV, Integer, 16, 1, 0),
    YUV444P16 = vs_make_video_id(YUV, Integer, 16, 0, 0),

    YUV420PH = vs_make_video_id(YUV, Float, 16, 1, 1),
    YUV420PS = vs_make_video_id(YUV, Float, 32, 1, 1),
    YUV422PH = vs_make_video_id(YUV, Float, 16, 1, 0),
    YUV422PS = vs_make_video_id(YUV, Float, 32, 1, 0),
    YUV444PH = vs_make_video_id(YUV, Float, 16, 0, 0),
    YUV444PS = vs_make_video_id(YUV, Float, 32, 0, 0),

    RGB24 = vs_make_video_id(RGB, Integer, 8, 0, 0),
    RGB27 = vs_make_video_id(RGB, Integer, 9, 0, 0),
    RGB30 = vs_make_video_id(RGB, Integer, 10, 0, 0),
    RGB36 = vs_make_video_id(RGB, Integer, 12, 0, 0),
    RGB42 = vs_make_video_id(RGB, Integer, 14, 0, 0),
    RGB48 = vs_make_video_id(RGB, Integer, 16, 0, 0),

    RGBH = vs_make_video_id(RGB, Float, 16, 0, 0),
    RGBS = vs_make_video_id(RGB, Float, 32, 0, 0),
}

/// Controls how a filter will be multithreaded, if at all.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSFilterMode {
    /// Completely parallel execution. Multiple threads will call a filter's "getFrame" function,
    /// to fetch several frames in parallel.
    Parallel = 0,
    /// For filters that are serial in nature but can request in advance one or more frames
    /// they need. A filter's "getFrame" function will be called from multiple threads at a time
    /// with activation reason [`VSActivationReason::Initial`],
    /// but only one thread will call it with activation reason
    /// [`VSActivationReason::AllFramesReady`] at a time.
    ParallelRequests = 1,
    /// Only one thread can call the filter's "getFrame" function at a time.
    /// Useful for filters that modify or examine their internal state to
    /// determine which frames to request.
    ///
    /// While the "getFrame" function will only run in one thread at a time,
    /// the calls can happen in any order. For example, it can be called with reason
    /// [`VSActivationReason::Initial`] for frame 0, then again with reason
    /// [`VSActivationReason::Initial`] for frame 1,
    /// then with reason [`VSActivationReason::AllFramesReady`]  for frame 0.
    Unordered = 2,
    /// For compatibility with other filtering architectures.
    /// *DO NOT USE IN NEW FILTERS*. The filter's "getFrame" function only ever gets called from
    /// one thread at a time. Unlike [`Unordered`](VSFilterMode::Unordered),
    /// only one frame is processed at a time.
    FrameState = 3,
}

/// Used to indicate the type of a [`VSFrame`] or [`VSNode`] object.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSMediaType {
    Video = 1,
    Audio = 2,
}

/// Describes the format of a clip.
///
/// Use [`queryVideoFormat()`](VSAPI::queryVideoFormat) to fill it in with proper error checking.
/// Manually filling out the struct is allowed but discouraged
/// since illegal combinations of values will cause undefined behavior.
#[repr(C)]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VSVideoFormat {
    /// See [`VSColorFamily`].
    pub color_family: VSColorFamily,
    /// See [`VSSampleType`].
    pub sample_type: VSSampleType,
    /// Number of significant bits.
    pub bits_per_sample: c_int,
    /// Number of bytes needed for a sample. This is always a power of 2 and the smallest possible
    /// that can fit the number of bits used per sample.
    pub bytes_per_sample: c_int,

    /// log2 subsampling factor, applied to second and third plane
    pub sub_sampling_w: c_int,
    /// log2 subsampling factor, applied to second and third plane.
    ///
    /// Convenient numbers that can be used like so:
    /// ```py
    /// uv_width = y_width >> subSamplingW;
    /// ```
    pub sub_sampling_h: c_int,

    /// Number of planes, implicit from colorFamily
    pub num_planes: c_int,
}

/// Audio channel positions as an enum. Mirrors the `FFmpeg` audio channel constants
/// in older api versions.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSAudioChannels {
    FrontLeft = 0,
    FrontRight = 1,
    FrontCenter = 2,
    LowFrequency = 3,
    BackLeft = 4,
    BackRight = 5,
    FrontLeftOFCenter = 6,
    FrontRightOFCenter = 7,
    BackCenter = 8,
    SideLeft = 9,
    SideRight = 10,
    TopCenter = 11,
    TopFrontLeft = 12,
    TopFrontCenter = 13,
    TopFrontRight = 14,
    TopBackLeft = 15,
    TopBackCenter = 16,
    TopBackRight = 17,
    StereoLeft = 29,
    StereoRight = 30,
    WideLeft = 31,
    WideRight = 32,
    SurroundDirectLeft = 33,
    SurroundDirectRight = 34,
    LowFrequency2 = 35,
}

/// Describes the format of a clip.
///
/// Use [`queryAudioFormat()`](VSAPI::queryAudioFormat) to fill it in with proper error checking.
/// Manually filling out the struct is allowed but discouraged
/// since illegal combinations of values will cause undefined behavior.
#[repr(C)]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VSAudioFormat {
    /// See [`VSSampleType`].
    pub sample_type: VSSampleType,
    /// Number of significant bits.
    pub bits_per_sample: c_int,
    /// Number of bytes needed for a sample. This is always a power of 2 and the smallest possible
    /// that can fit the number of bits used per sample, implicit from
    /// [`VSAudioFormat::channel_layout`].
    pub bytes_per_sample: c_int,
    /// Number of audio channels, implicit from [`VSAudioFormat::bits_per_sample`]
    pub num_channels: c_int,
    /// A bitmask representing the channels present using the constants in 1 left shifted
    /// by the constants in [`VSAudioChannels`].
    pub channel_layout: u64,
}

/// Types of properties that can be stored in a [`VSMap`].
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSPropertyType {
    Unset = 0,
    Int = 1,
    Float = 2,
    Data = 3,
    Function = 4,
    VideoNode = 5,
    AudioNode = 6,
    VideoFrame = 7,
    AudioFrame = 8,
}

/// When a `mapGet*` function fails, it returns one of these in the err parameter.
///
/// All errors are non-zero.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSMapPropertyError {
    Success = 0,
    /// The requested key was not found in the map.
    Unset = 1,
    /// The wrong function was used to retrieve the property.
    /// E.g. [`mapGetInt()`](VSAPI::mapGetInt) was used on a property of type
    /// [`VSPropertyType::Float`].
    Type = 2,
    /// The requested index was out of bounds.
    Index = 4,
    /// The map has the error state set.
    Error = 3,
}

/// Controls the behaviour of [`mapSetInt()`](VSAPI::mapSetInt) and friends.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSMapAppendMode {
    /// All existing values associated with the key will be replaced with the new value.
    Replace = 0,
    /// The new value will be appended to the list of existing values associated with the key.
    Append = 1,
}

/// Contains information about a [`VSCore`] instance.
#[repr(C)]
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct VSCoreInfo {
    /// Printable string containing the name of the library, copyright notice,
    /// core and API versions.
    pub version_string: *const c_char,
    /// Version of the core.
    pub core: c_int,
    /// Version of the API.
    pub api: c_int,
    /// Number of worker threads.
    pub num_threads: c_int,
    /// The framebuffer cache will be allowed to grow up to this size (bytes)
    /// before memory is aggressively reclaimed.
    pub max_framebuffer_size: i64,
    /// Current size of the framebuffer cache, in bytes.
    pub used_framebuffer_size: i64,
}

/// Contains information about a clip.
#[repr(C)]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VSVideoInfo {
    /// Format of the clip. Will have [`VSVideoFormat::color_family`] set to
    /// [`VSColorFamily::Undefined`] if the format can vary.
    pub format: VSVideoFormat,
    /// Numerator part of the clip's frame rate. It will be 0 if the frame rate can vary.
    /// Should always be a reduced fraction.
    pub fps_num: i64,
    /// Denominator part of the clip's frame rate. It will be 0 if the frame rate can vary.
    /// Should always be a reduced fraction.
    pub fps_den: i64,
    /// Width of the clip. Both width and height will be 0 if the clip's dimensions can vary.
    pub width: c_int,
    /// Height of the clip. Both width and height will be 0 if the clip's dimensions can vary.
    pub height: c_int,
    /// Length of the clip.
    pub num_frames: c_int,
}

/// Contains information about a clip.
#[repr(C)]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VSAudioInfo {
    /// Format of the clip. Unlike video the audio format can never change.
    pub format: VSAudioFormat,
    /// Sample rate.
    pub sample_rate: c_int,
    /// Length of the clip in audio samples.
    pub num_samples: i64,
    /// Length of the clip in audio frames.
    ///
    /// The total number of audio frames needed to hold [`Self::num_samples`],
    /// implicit from [`Self::num_samples`] when calling
    /// [`createAudioFilter()`](VSAPI::createAudioFilter)
    pub num_frames: c_int,
}

/// See [`VSFilterGetFrame`].
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSActivationReason {
    Initial = 0,
    AllFramesReady = 1,
    Error = -1,
}

/// See [`addLogHandler()`](VSAPI::addLogHandler).
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSMessageType {
    Debug = 0,
    Information = 1,
    Warning = 2,
    Critical = 3,
    /// also terminates the process, should generally not be used by normal filters
    Fatal = 4,
}

/// Options when creating a core.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSCoreCreationFlags {
    /// Required to use the graph inspection api functions.
    /// Increases memory usage due to the extra information stored.
    EnableGraphInspection = 1,
    /// Don't autoload any user plugins. Core plugins are always loaded.
    DisableAutoLoading = 2,
    /// Don't unload plugin libraries when the core is destroyed.
    /// Due to a small amount of memory leaking every load and unload
    /// (windows feature, not my fault) of a library,
    /// this may help in applications with extreme amount of script reloading.
    DisableLibraryUnloading = 4,
}

impl std::ops::BitOr for VSCoreCreationFlags {
    type Output = c_int;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as c_int | rhs as c_int
    }
}

/// Options when loading a plugin.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSPluginConfigFlags {
    /// Allow functions to be added to the plugin object after the plugin loading phase.
    /// Mostly useful for Avisynth compatibility and other foreign plugin loaders.
    Modifiable = 1,
}

impl std::ops::BitOr for VSPluginConfigFlags {
    type Output = c_int;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as c_int | rhs as c_int
    }
}

/// Since the data type can contain both pure binary data and printable strings,
/// the type also contains a hint for whether or not it is human readable.
/// Generally the unknown type should be very rare and is almost only created
/// as an artifact of API3 compatibility.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSDataTypeHint {
    Unknown = -1,
    Binary = 0,
    Utf8 = 1,
}

/// Describes the upstream frame request pattern of a filter.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSRequestPattern {
    /// Anything goes. Note that filters that may be requesting beyond the end of a
    /// [`VSNode`] length in frames (repeating the last frame) should use
    /// [`VSRequestPattern::General`]) and not any of the other modes.
    General = 0,
    /// Will only request an input frame at most once if all output frames are requested
    /// exactly one time. This includes filters such as Trim, Reverse, `SelectEvery`.
    NoFrameReuse = 1,
    /// Only requests frame N to output frame N. The main difference to
    /// [`VSRequestPattern::NoFrameReuse`] is that the requested frame
    /// is always fixed and known ahead of time. Filter examples
    /// Lut, Expr (conditionally, see [`VSRequestPattern::General`] note)
    /// and similar.
    StrictSpatial = 2,
}

/// Describes how the output of a node is cached.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSCacheMode {
    /// Cache is enabled or disabled based on the reported request patterns
    /// and number of consumers.
    Auto = -1,
    /// Never cache anything.
    ForceDisable = 0,
    /// Always use the cache.
    ForceEnable = 1,
}

/// Core entry point
pub type VSGetVapourSynthAPI = unsafe extern "system-unwind" fn(version: c_int) -> *const VSAPI;

// SECTION - Plugin, function and filter related
/// User-defined function called by the core to create an instance of the filter.
/// This function is often named `fooCreate`.
///
/// In this function, the filter's input parameters should be retrieved and validated,
/// the filter's private instance data should be initialised, and
/// [`createAudioFilter()`](VSAPI::createAudioFilter) or
/// [`createVideoFilter()`](VSAPI::createVideoFilter) should be called.
/// This is where the filter should perform any other initialisation it requires.
///
/// If for some reason you cannot create the filter, you have to free any created node references
/// using [`freeNode()`](VSAPI::freeNode), call [`mapSetError()`](VSAPI::mapSetError) on `out`,
/// and return.
///
/// # Arguments
///
/// * `in` - Input parameter list.
///
///     Use [`mapGetInt()`](VSAPI::mapGetInt) and friends to retrieve a parameter value.
///
///     The map is guaranteed to exist only until the filter's "init" function returns.
///     In other words, pointers returned by [`mapGetData()`](VSAPI::mapGetData)
///     will not be usable in the filter's "getFrame" and "free" functions.
///
/// * `out` - Output parameter list. [`createAudioFilter()`](VSAPI::createAudioFilter) or
///     [`createVideoFilter()`](VSAPI::createVideoFilter) will add the output node(s)
///     with the key named "clip", or an error, if something went wrong.
///
/// * `userData` - Pointer that was passed to [`registerFunction()`](VSAPI::registerFunction).
pub type VSPublicFunction = unsafe extern "system-unwind" fn(
    in_: *const VSMap,
    out: *mut VSMap,
    userData: *mut c_void,
    core: *mut VSCore,
    vsapi: *const VSAPI,
);
/// A plugin's entry point. It must be called `VapourSynthPluginInit2`.
/// This function is called after the core loads the shared library.
/// Its purpose is to configure the plugin and to register the filters the plugin wants to export.
///
/// # Arguments
///
/// * `plugin` - A pointer to the plugin object to be initialized.
/// * `vspapi` - A pointer to a [`VSPLUGINAPI`] struct with a subset of the `VapourSynth` API
///     used for initializing plugins. The proper way to do things is to call
///     [`configPlugin`](VSPLUGINAPI::configPlugin) and then
///     [`registerFunction`](VSPLUGINAPI::registerFunction) for each function to export.
pub type VSInitPlugin =
    unsafe extern "system-unwind" fn(plugin: *mut VSPlugin, vspapi: *const VSPLUGINAPI);
/// Free function type
pub type VSFreeFunctionData = Option<unsafe extern "system-unwind" fn(userData: *mut c_void)>;
/// A filter's "getFrame" function. It is called by the core when it needs the filter
/// to generate a frame.
///
/// It is possible to allocate local data, persistent during the multiple calls
/// requesting the output frame.
///
/// In case of error, call [`setFilterError()`](VSAPI::setFilterError),
/// free `*frameData` if required, and return `NULL`.
///
/// Depending on the [`VSFilterMode`] set for the filter, multiple output frames
/// could be requested concurrently.
///
/// It is never called concurrently for the same frame number.
///
/// # Arguments
///
/// * `n` - Requested frame number.
/// * `activationReason` - One of [`VSActivationReason`].
///
///     ## Note
///
///     This function is first called with [`VSActivationReason::Initial`].
///     At this point the function should request the input frames it needs and return `NULL`.
///     When one or all of the requested frames are ready, this function is called again with
///     [`VSActivationReason::AllFramesReady`].
///     The function should only return a frame when called with
///     [`VSActivationReason::AllFramesReady`].
///
///     If a the function is called with [`VSActivationReason::Error`] all processing has
///     to be aborted and any.
///
/// * `instanceData` - The filter's private instance data.
/// * `frameData` - Optional private data associated with output frame number `n`.
///     It must be deallocated before the last call for the given frame
///     ([`VSActivationReason::AllFramesReady`] or error).
///
///     It points to a `void *[4]` array of memory that may be used freely.
///     See filters like Splice and Trim for examples.
///
/// Return a reference to the output frame number n when it is ready, or `NULL`.
/// The ownership of the frame is transferred to the caller.
pub type VSFilterGetFrame = unsafe extern "system-unwind" fn(
    n: c_int,
    activationReason: VSActivationReason,
    instanceData: *mut c_void,
    frameData: *mut *mut c_void,
    frameCtx: *mut VSFrameContext,
    core: *mut VSCore,
    vsapi: *const VSAPI,
) -> *const VSFrame;
/// A filter's "free" function.
///
/// This is where the filter should free everything it allocated, including its instance data.
//
/// # Arguments
///
/// * `instanceData` - The filter's private instance data.
pub type VSFilterFree = Option<
    unsafe extern "system-unwind" fn(
        instanceData: *mut c_void,
        core: *mut VSCore,
        vsapi: *const VSAPI,
    ),
>;
// !SECTION

// SECTION - Other
/// Function of the client application called by the core when a requested frame is ready,
/// after a call to [`getFrameAsync()`](VSAPI::getFrameAsync).
///
/// If multiple frames were requested, they can be returned in any order.
/// Client applications must take care of reordering them.
///
/// This function is only ever called from one thread at a time.
///
/// [`getFrameAsync()`](VSAPI::getFrameAsync) may be called from this function to
/// request more frames.
///
/// # Arguments
///
/// * `userData` - Pointer to private data from the client application,
///     as passed previously to [`getFrameAsync()`](VSAPI::getFrameAsync).
///
/// * `f` - Contains a reference to the generated frame, or `NULL` in case of failure.
///     The ownership of the frame is transferred to the caller.
///
/// * `n` - The frame number.
///
/// * `node` - Node the frame belongs to.
///
/// * `errorMsg` - String that usually contains an error message if the frame generation failed.
///     `NULL` if there is no error.
pub type VSFrameDoneCallback = unsafe extern "system-unwind" fn(
    userData: *mut c_void,
    f: *const VSFrame,
    n: c_int,
    node: *mut VSNode,
    errorMsg: *const c_char,
);
/// # Arguments
///
/// * `msgType` - The type of message. One of [`VSMessageType`].
///
///     If `msgType` is [`VSMessageType::Fatal`]),
///     `VapourSynth` will call `abort()` after the message handler returns.
///
/// * `msg` - The message.
pub type VSLogHandler = Option<
    unsafe extern "system-unwind" fn(msgType: c_int, msg: *const c_char, userData: *mut c_void),
>;
pub type VSLogHandlerFree = Option<unsafe extern "system-unwind" fn(userData: *mut c_void)>;
// !SECTION

/// This struct is used to access `VapourSynth`'s API when a plugin is initially loaded.
#[allow(non_snake_case)]
#[repr(C)]
pub struct VSPLUGINAPI {
    /// See [`getAPIVersion()`](VSAPI::getAPIVersion) in the struct [`VSAPI`].
    /// Returns [`VAPOURSYNTH_API_VERSION`] of the library
    pub getAPIVersion: unsafe extern "system-unwind" fn() -> c_int,
    /// Used to provide information about a plugin when loaded. Must be called exactly once from
    /// the `VapourSynthPluginInit2()` entry point. It is recommended to use the
    /// [`vs_make_version]` macro when providing the `pluginVersion`.
    /// If you don't know the specific `apiVersion` you actually require simply pass
    /// [`VAPOURSYNTH_API_VERSION`] to match the header version
    /// you're compiling against. The flags consist of values from
    /// [`VSPluginConfigFlags`] `ORed` together but should for most plugins typically be 0.
    ///
    /// Returns non-zero on success.
    pub configPlugin: unsafe extern "system-unwind" fn(
        identifier: *const c_char,
        pluginNamespace: *const c_char,
        name: *const c_char,
        pluginVersion: c_int,
        apiVersion: c_int,
        flags: c_int,
        plugin: *mut VSPlugin,
    ) -> c_int,
    /// See [`registerFunction()`](VSAPI::registerFunction) in the struct [`VSAPI`],
    ///
    /// Returns non-zero on success.
    pub registerFunction: unsafe extern "system-unwind" fn(
        name: *const c_char,
        args: *const c_char,
        returnType: *const c_char,
        argsFunc: VSPublicFunction,
        functionData: *mut c_void,
        plugin: *mut VSPlugin,
    ) -> c_int,
}

/// Specifies the dependency of a filter on other nodes.
#[repr(C)]
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct VSFilterDependency {
    /// The node frames are requested from.
    pub source: *mut VSNode,
    /// A value from [`VSRequestPattern`].
    pub request_pattern: VSRequestPattern,
}

// MARK: VSAPI

/// This giant struct is the way to access `VapourSynth`'s public API.
#[allow(non_snake_case)]
#[repr(C)]
pub struct VSAPI {
    // SECTION - Audio and video filter related including nodes
    /// Creates a new video filter node.
    ///
    /// # Arguments
    ///
    /// * `out` - Output map for the filter node.
    ///
    /// * `name` - Instance name. Please make it the same as
    ///     the filter's name for easy identification.
    ///
    /// * `vi` - The output format of the filter.
    ///
    /// * `getFrame` - The filter's "getFrame" function. Must not be `NULL`.
    ///
    /// * `free` - The filter's "free" function. Can be `NULL`.
    ///
    /// * `filterMode` - One of [`VSFilterMode`].
    ///     Indicates the level of parallelism supported by the filter.
    ///
    /// * `dependencies` - An array of nodes the filter requests frames from
    ///     and the access pattern. Used to more efficiently configure caches.
    ///
    /// * `numDeps` - Length of the dependencies array.
    ///
    /// * `instanceData` - A pointer to the private filter data. This pointer will be passed to
    ///     the `getFrame` and `free` functions. It should be freed by the free function.
    ///
    /// After this function returns, `out` will contain the new node appended to
    /// the "clip" property, or an error, if something went wrong.
    pub createVideoFilter: unsafe extern "system-unwind" fn(
        out: *mut VSMap,
        name: *const c_char,
        vi: *const VSVideoInfo,
        getFrame: VSFilterGetFrame,
        free: VSFilterFree,
        filterMode: VSFilterMode,
        dependencies: *const VSFilterDependency,
        numDeps: c_int,
        instanceData: *mut c_void,
        core: *mut VSCore,
    ),
    /// Identical to [`createVideoFilter()`](Self::createVideoFilter) except that
    /// the new node is returned instead of appended to the out map.
    ///
    /// Returns `NULL` on error.
    pub createVideoFilter2: unsafe extern "system-unwind" fn(
        name: *const c_char,
        vi: *const VSVideoInfo,
        getFrame: VSFilterGetFrame,
        free: VSFilterFree,
        filterMode: VSFilterMode,
        dependencies: *const VSFilterDependency,
        numDeps: c_int,
        instanceData: *mut c_void,
        core: *mut VSCore,
    ) -> *mut VSNode,
    /// Creates a new video filter node.
    ///
    /// # Arguments
    ///
    /// * `out` - Output map for the filter node.
    ///
    /// * `name` - Instance name. Please make it the same as
    ///     the filter's name for easy identification.
    ///
    /// * `ai` - The output format of the filter.
    ///
    /// * `getFrame` - The filter's "getFrame" function. Must not be `NULL`.
    ///
    /// * `free` - The filter's "free" function. Can be `NULL`.
    ///
    /// * `filterMode` - One of [`VSFilterMode`].
    ///     Indicates the level of parallelism supported by the filter.
    ///
    /// * `dependencies` - An array of nodes the filter requests frames from
    ///     and the access pattern. Used to more efficiently configure caches.
    ///
    /// * `numDeps` - Length of the dependencies array.
    ///
    /// * `instanceData` - A pointer to the private filter data. This pointer will be passed to
    ///     the `getFrame` and `free` functions. It should be freed by the free function.
    ///
    /// After this function returns, out will contain the new node appended to
    /// the "clip" property, or an error, if something went wrong.
    pub createAudioFilter: unsafe extern "system-unwind" fn(
        out: *mut VSMap,
        name: *const c_char,
        ai: *const VSAudioInfo,
        getFrame: VSFilterGetFrame,
        free: VSFilterFree,
        filterMode: VSFilterMode,
        dependencies: *const VSFilterDependency,
        numDeps: c_int,
        instanceData: *mut c_void,
        core: *mut VSCore,
    ),
    /// Identical to [`createAudioFilter()`](Self::createAudioFilter) except that
    /// the new node is returned instead of appended to the out map.
    ///
    /// Returns `NULL` on error.
    pub createAudioFilter2: unsafe extern "system-unwind" fn(
        name: *const c_char,
        ai: *const VSAudioInfo,
        getFrame: VSFilterGetFrame,
        free: VSFilterFree,
        filterMode: VSFilterMode,
        dependencies: *const VSFilterDependency,
        numDeps: c_int,
        instanceData: *mut c_void,
        core: *mut VSCore,
    ) -> *mut VSNode,
    /// Must be called immediately after audio or video filter creation.
    ///
    /// Returns the upper bound of how many additional frames it is reasonable to pass to
    /// [`cacheFrame()`](Self::cacheFrame) when trying to make a request more linear.
    pub setLinearFilter: unsafe extern "system-unwind" fn(node: *mut VSNode) -> c_int,
    /// Determines the strategy for frame caching. Pass a [`VSCacheMode`] constant.
    /// Mostly useful for cache debugging since the auto mode should
    /// work well in just about all cases. Calls to this function may also be silently ignored.
    ///
    /// Resets the cache to default options when called, discarding
    /// [`setCacheOptions`](Self::setCacheOptions) changes.
    pub setCacheMode: unsafe extern "system-unwind" fn(node: *mut VSNode, mode: VSCacheMode),
    /// Call after setCacheMode or the changes will be discarded.
    /// Sets internal details of a node's associated cache.
    /// Calls to this function may also be silently ignored.
    ///
    /// # Arguments
    ///
    /// * `fixedSize` - Set to non-zero to make the cache always hold `maxSize` frames.
    ///
    /// * `maxSize` - The maximum number of frames to cache.
    ///     Note that this value is automatically adjusted using
    ///     an internal algorithm unless fixedSize is set.
    ///
    /// * `maxHistorySize` - How many frames that have been recently evicted from the cache to
    ///     keep track off. Used to determine if growing or shrinking the cache is beneficial.
    ///     Has no effect when `fixedSize` is set.
    pub setCacheOptions: unsafe extern "system-unwind" fn(
        node: *mut VSNode,
        fixedSize: c_int,
        maxSize: c_int,
        maxHistorySize: c_int,
    ),

    /// Decreases the reference count of a node and destroys it once it reaches 0.
    ///
    /// It is safe to pass `NULL`.
    pub freeNode: unsafe extern "system-unwind" fn(node: *mut VSNode),
    /// Increment the reference count of a node. Returns the same node for convenience.
    pub addNodeRef: unsafe extern "system-unwind" fn(node: *mut VSNode) -> *mut VSNode,
    /// Returns [`VSMediaType`]. Used to determine if a node is of audio or video type.
    pub getNodeType: unsafe extern "system-unwind" fn(node: *mut VSNode) -> VSMediaType,
    /// Returns a pointer to the video info associated with a node.
    /// The pointer is valid as long as the node lives.
    /// It is undefined behavior to pass a non-video node.
    pub getVideoInfo: unsafe extern "system-unwind" fn(node: *mut VSNode) -> *const VSVideoInfo,
    /// Returns a pointer to the audio info associated with a node.
    /// The pointer is valid as long as the node lives.
    /// It is undefined behavior to pass a non-audio node.
    pub getAudioInfo: unsafe extern "system-unwind" fn(node: *mut VSNode) -> *const VSAudioInfo,
    // !SECTION

    // SECTION - Frame related functions
    /// Creates a new video frame, optionally copying the properties attached to another frame.
    /// It is a fatal error to pass invalid arguments to this function.
    ///
    /// The new frame contains uninitialised memory.
    ///
    /// # Arguments
    ///
    /// * `format` - The desired colorspace format. Must not be `NULL`.
    ///
    /// * `width` -
    /// * `height` - The desired dimensions of the frame, in pixels.
    ///     Must be greater than 0 and have a suitable multiple for the subsampling in format.
    ///
    /// * `propSrc` - A frame from which properties will be copied. Can be `NULL`.
    ///
    /// Returns a pointer to the created frame.
    /// Ownership of the new frame is transferred to the caller.
    ///
    /// See also [`newVideoFrame2()`](Self::newVideoFrame2).
    pub newVideoFrame: unsafe extern "system-unwind" fn(
        format: *const VSVideoFormat,
        width: c_int,
        height: c_int,
        propSrc: *const VSFrame,
        core: *mut VSCore,
    ) -> *mut VSFrame,
    /// Creates a new video frame, optionally copying the properties attached to another frame.
    /// It is a fatal error to pass invalid arguments to this function.
    ///
    /// The new frame contains uninitialised memory.
    ///
    /// # Arguments
    ///
    /// * `format` - The desired colorspace format. Must not be `NULL`.
    ///
    /// * `width` -
    /// * `height` - The desired dimensions of the frame, in pixels.
    ///     Must be greater than 0 and have a suitable multiple for the subsampling in format.
    ///
    /// * `planeSrc` - Array of frames from which planes will be copied.
    ///     If any elements of the array are `NULL`, the corresponding planes in the new frame
    ///     will contain uninitialised memory.
    ///
    /// * `planes` - Array of plane numbers indicating which plane to copy from
    ///     the corresponding source frame.
    ///
    /// * `propSrc` - A frame from which properties will be copied. Can be `NULL`.
    ///
    /// Returns a pointer to the created frame.
    /// Ownership of the new frame is transferred to the caller.
    ///
    /// # Example
    ///
    /// (assume frameA, frameB, frameC are existing frames):
    ///
    /// ```c
    /// const VSFrame * frames[3] = { frameA, frameB, frameC };
    /// const int planes[3] = { 1, 0, 2 };
    /// VSFrame *newFrame = vsapi->newVideoFrame2(f, w, h, frames, planes, frameB, core);
    /// ```
    ///
    /// The newFrame's first plane is now a copy of frameA's second plane,
    /// the second plane is a copy of frameB's first plane,
    /// the third plane is a copy of frameC's third plane
    /// and the properties have been copied from frameB.
    pub newVideoFrame2: unsafe extern "system-unwind" fn(
        format: *const VSVideoFormat,
        width: c_int,
        height: c_int,
        planeSrc: *const *const VSFrame,
        planes: *const c_int,
        propSrc: *const VSFrame,
        core: *mut VSCore,
    ) -> *mut VSFrame,
    /// Creates a new audio frame, optionally copying the properties attached to another frame.
    /// It is a fatal error to pass invalid arguments to this function.
    ///
    /// The new frame contains uninitialised memory.
    ///
    /// # Arguments
    ///
    /// * `format` - The desired audio format. Must not be `NULL`.
    ///
    /// * `numSamples` - The number of samples in the frame. All audio frames apart from
    ///     the last one returned by a filter must have [`VS_AUDIO_FRAME_SAMPLES`].
    ///
    /// * `propSrc` - A frame from which properties will be copied. Can be `NULL`.
    ///
    /// Returns a pointer to the created frame.
    /// Ownership of the new frame is transferred to the caller.
    ///
    /// See also [`newAudioFrame2()`](Self::newAudioFrame2).
    pub newAudioFrame: unsafe extern "system-unwind" fn(
        format: *const VSAudioFormat,
        numSamples: c_int,
        propSrc: *const VSFrame,
        core: *mut VSCore,
    ) -> *mut VSFrame,
    /// Creates a new audio frame, optionally copying the properties attached to another frame.
    /// It is a fatal error to pass invalid arguments to this function.
    ///
    /// The new frame contains uninitialised memory.
    ///
    /// # Arguments
    ///
    /// * `format` - The desired audio format. Must not be `NULL`.
    ///
    /// * `numSamples` - The number of samples in the frame. All audio frames apart from
    ///     the last one returned by a filter must have [`VS_AUDIO_FRAME_SAMPLES`].
    ///
    /// * `propSrc` - A frame from which properties will be copied. Can be `NULL`.
    ///
    /// * `channelSrc` - Array of frames from which channels will be copied.
    ///     If any elements of the array are `NULL`, the corresponding planes in
    ///     the new frame will contain uninitialised memory.
    ///
    /// * `channels` - Array of channel numbers indicating which channel to copy from
    ///     the corresponding source frame.
    ///     Note that the number refers to the nth channel and not a channel name constant.
    ///
    /// Returns a pointer to the created frame.
    /// Ownership of the new frame is transferred to the caller.
    pub newAudioFrame2: unsafe extern "system-unwind" fn(
        format: *const VSAudioFormat,
        numSamples: c_int,
        channelSrc: *const *const VSFrame,
        channels: *const c_int,
        propSrc: *const VSFrame,
        core: *mut VSCore,
    ) -> *mut VSFrame,
    /// Decrements the reference count of a frame and deletes it when it reaches 0.
    ///
    /// It is safe to pass `NULL`.
    pub freeFrame: unsafe extern "system-unwind" fn(f: *const VSFrame),
    /// Increments the reference count of a frame. Returns f as a convenience.
    pub addFrameRef: unsafe extern "system-unwind" fn(f: *const VSFrame) -> *mut VSFrame,
    /// Duplicates the frame (not just the reference). As the frame buffer is shared in
    /// a copy-on-write fashion, the frame content is not really duplicated until
    /// a write operation occurs. This is transparent for the user.
    ///
    /// Returns a pointer to the new frame. Ownership is transferred to the caller.
    pub copyFrame:
        unsafe extern "system-unwind" fn(f: *const VSFrame, core: *mut VSCore) -> *mut VSFrame,
    /// Returns a read-only pointer to a frame's properties.
    /// The pointer is valid as long as the frame lives.
    pub getFramePropertiesRO: unsafe extern "system-unwind" fn(f: *const VSFrame) -> *const VSMap,
    /// Returns a read/write pointer to a frame's properties.
    /// The pointer is valid as long as the frame lives.
    pub getFramePropertiesRW: unsafe extern "system-unwind" fn(f: *mut VSFrame) -> *mut VSMap,

    /// Returns the distance in bytes between two consecutive lines of a plane of a video frame.
    /// The stride is always positive.
    ///
    /// Returns 0 if the requested plane doesn't exist or if it isn't a video frame.
    pub getStride: unsafe extern "system-unwind" fn(f: *const VSFrame, plane: c_int) -> isize,
    /// Returns a read-only pointer to a plane or channel of a frame.
    /// Returns `NULL` if an invalid plane or channel number is passed.
    ///
    /// # Note
    ///
    /// Don't assume all three planes of a frame are allocated
    /// in one contiguous chunk (they're not).
    pub getReadPtr: unsafe extern "system-unwind" fn(f: *const VSFrame, plane: c_int) -> *const u8,
    /// Returns a read-write pointer to a plane or channel of a frame.
    /// Returns `NULL` if an invalid plane or channel number is passed.
    ///
    /// # Note
    ///
    /// Don't assume all three planes of a frame are allocated
    /// in one contiguous chunk (they're not).
    pub getWritePtr: unsafe extern "system-unwind" fn(f: *mut VSFrame, plane: c_int) -> *mut u8,

    /// Retrieves the format of a video frame.
    pub getVideoFrameFormat:
        unsafe extern "system-unwind" fn(f: *const VSFrame) -> *const VSVideoFormat,
    /// Retrieves the format of an audio frame.
    pub getAudioFrameFormat:
        unsafe extern "system-unwind" fn(f: *const VSFrame) -> *const VSAudioFormat,
    /// Returns a value from [`VSMediaType`] to distinguish audio and video frames.
    pub getFrameType: unsafe extern "system-unwind" fn(f: *const VSFrame) -> VSMediaType,
    /// Returns the width of a plane of a given video frame, in pixels.
    /// The width depends on the plane number because of the possible chroma subsampling.
    ///
    /// Returns 0 for audio frames.
    pub getFrameWidth: unsafe extern "system-unwind" fn(f: *const VSFrame, plane: c_int) -> c_int,
    /// Returns the height of a plane of a given video frame, in pixels.
    /// The height depends on the plane number because of the possible chroma subsampling.
    ///
    /// Returns 0 for audio frames.
    pub getFrameHeight: unsafe extern "system-unwind" fn(f: *const VSFrame, plane: c_int) -> c_int,
    /// Returns the number of audio samples in a frame. Always returns 1 for video frames.
    pub getFrameLength: unsafe extern "system-unwind" fn(f: *const VSFrame) -> c_int,
    // !SECTION

    // SECTION - General format functions
    /// Tries to output a fairly human-readable name of a video format.
    ///
    /// # Arguments
    ///
    /// * `format` - The input video format.
    /// * `buffer` - Destination buffer. At most 32 bytes including
    ///     terminating `NUL` will be written.
    ///
    /// Returns non-zero on success.
    pub getVideoFormatName: unsafe extern "system-unwind" fn(
        format: *const VSVideoFormat,
        buffer: *mut c_char,
    ) -> c_int,
    /// Tries to output a fairly human-readable name of an audio format.
    ///
    /// # Arguments
    ///
    /// * `format` - The input audio format.
    /// * `buffer` - Destination buffer. At most 32 bytes including
    ///     terminating `NUL` will be written.
    ///
    /// Returns non-zero on success.
    pub getAudioFormatName: unsafe extern "system-unwind" fn(
        format: *const VSAudioFormat,
        buffer: *mut c_char,
    ) -> c_int,
    /// Fills out a \[_sic_\] [`VSVideoInfo`] struct based on the provided arguments.
    /// Validates the arguments before filling out format.
    ///
    /// # Arguments
    ///
    /// * `format` - The struct to fill out.
    /// * `colorFamily` - One of [`VSColorFamily`].
    /// * `sampleType` - One of [`VSSampleType`].
    /// * `bitsPerSample` - Number of meaningful bits for a single component.
    ///     The valid range is 8-32.
    ///
    ///     For floating point formats only 16 or 32 bits are allowed.
    /// * `subSamplingW` - log2 of the horizontal chroma subsampling.
    ///     0 == no subsampling. The valid range is 0-4.
    /// * `subSamplingH` - log2 of the vertical chroma subsampling.
    ///     0 == no subsampling. The valid range is 0-4.
    ///
    ///     ## Note
    ///     
    ///     RGB formats are not allowed to be subsampled in `VapourSynth`.
    ///
    /// Returns non-zero on success.
    pub queryVideoFormat: unsafe extern "system-unwind" fn(
        format: *mut VSVideoFormat,
        colorFamily: VSColorFamily,
        sampleType: VSSampleType,
        bitsPerSample: c_int,
        subSamplingW: c_int,
        subSamplingH: c_int,
        core: *mut VSCore,
    ) -> c_int,
    /// Fills out a [`VSAudioFormat`] struct based on the provided arguments.
    /// Validates the arguments before filling out format.
    ///
    /// # Arguments
    ///
    /// * `format` - The struct to fill out.
    ///
    /// * `sampleType` - One of [`VSSampleType`].
    ///
    /// * `bitsPerSample` - Number of meaningful bits for a single component.
    ///     The valid range is 8-32.
    ///
    ///     For floating point formats only 32 bits are allowed.
    ///
    /// * `channelLayout` - A bitmask constructed from bitshifted constants in
    ///     [`VSAudioChannels`]. For example stereo is expressed as
    ///     `(1 << acFrontLeft) | (1 << acFrontRight)`.
    ///
    /// Returns non-zero on success.
    pub queryAudioFormat: unsafe extern "system-unwind" fn(
        format: *mut VSAudioFormat,
        sampleType: VSSampleType,
        bitsPerSample: c_int,
        channelLayout: u64,
        core: *mut VSCore,
    ) -> c_int,
    /// Get the id associated with a video format. Similar to
    /// [`queryVideoFormat()`](Self::queryVideoFormat) except that it returns a format id
    /// instead of filling out a [`VSVideoInfo`] struct.
    ///
    /// # Arguments
    ///
    /// * `colorFamily` - One of [`VSColorFamily`].
    ///
    /// * `sampleType` - One of [`VSSampleType`].
    ///
    /// * `bitsPerSample` - Number of meaningful bits for a single component.
    ///     The valid range is 8-32.
    ///
    ///     For floating point formats only 16 or 32 bits are allowed.
    ///
    /// * `subSamplingW` - log2 of the horizontal chroma subsampling.
    ///     0 == no subsampling. The valid range is 0-4.
    ///
    /// * `subSamplingH` - log2 of the vertical chroma subsampling.
    ///     0 == no subsampling. The valid range is 0-4.
    ///
    ///     ## Note
    ///     
    ///     RGB formats are not allowed to be subsampled in `VapourSynth`.
    ///
    /// Returns a valid format id if the provided arguments are valid, on error 0 is returned.
    pub queryVideoFormatID: unsafe extern "system-unwind" fn(
        colorFamily: VSColorFamily,
        sampleType: VSSampleType,
        bitsPerSample: c_int,
        subSamplingW: c_int,
        subSamplingH: c_int,
        core: *mut VSCore,
    ) -> u32,
    /// Fills out the `VSVideoFormat` struct passed to format based
    ///
    /// # Arguments
    ///
    /// * `format` - The struct to fill out.
    ///
    /// * `id` - The format identifier: one of [`VSPresetVideoFormat`]
    ///     or a value gotten from [`queryVideoFormatID()`](Self::queryVideoFormatID).
    ///
    /// Returns 0 on failure and non-zero on success.
    pub getVideoFormatByID: unsafe extern "system-unwind" fn(
        format: *mut VSVideoFormat,
        id: u32,
        core: *mut VSCore,
    ) -> c_int,
    // !SECTION

    // SECTION - Frame request and filter getFrame functions
    /// Fetches a frame synchronously. The frame is available when the function returns.
    ///
    /// This function is meant for external applications using the core as a library,
    /// or if frame requests are necessary during a filter's initialization.
    ///
    /// Thread-safe.
    ///
    /// # Arguments
    ///
    /// * `n` - The frame number. Negative values will cause an error.
    ///
    /// * `node` - The node from which the frame is requested.
    ///
    /// * `errorMsg` - Pointer to a buffer of `bufSize` bytes to store a possible error message.
    ///     Can be `NULL` if no error message is wanted.
    ///
    /// * `bufSize` - Maximum length for the error message, in bytes (including the trailing '0').
    ///     Can be 0 if no error message is wanted.
    ///
    /// Returns a reference to the generated frame, or `NULL` in case of failure.
    /// The ownership of the frame is transferred to the caller.
    ///
    /// # Warning
    ///
    /// Never use inside a filter's "getFrame" function.
    pub getFrame: unsafe extern "system-unwind" fn(
        n: c_int,
        node: *mut VSNode,
        errorMsg: *mut c_char,
        bufSize: c_int,
    ) -> *const VSFrame,
    /// Requests the generation of a frame. When the frame is ready,
    /// a user-provided function is called.
    /// Note that the completion callback will only be called from a single thread at a time.
    ///
    /// This function is meant for applications using `VapourSynth` as a library.
    ///
    /// Thread-safe.
    ///
    /// # Arguments
    ///
    /// * `n` - Frame number. Negative values will cause an error.
    ///
    /// * `node` - The node from which the frame is requested.
    ///
    /// * `callback` - See [`VSFrameDoneCallback`].
    ///
    /// * `userData` - Pointer passed to the callback.
    ///
    /// # Warning
    ///
    /// Never use inside a filter's "getFrame" function.
    pub getFrameAsync: unsafe extern "system-unwind" fn(
        n: c_int,
        node: *mut VSNode,
        callback: VSFrameDoneCallback,
        userData: *mut c_void,
    ),
    /// Retrieves a frame that was previously requested with
    /// [`requestFrameFilter()`](Self::requestFrameFilter).
    ///
    /// Only use inside a filter's "getFrame" function.
    ///
    /// A filter usually calls this function when its activation reason is
    /// [`VSActivationReason::AllFramesReady`].
    /// See [`VSActivationReason`].
    ///
    /// It is safe to retrieve a frame more than once, but each reference needs to be freed.
    ///
    /// # Arguments
    ///
    /// * `n` - The frame number.
    ///
    /// * `node` - The node from which the frame is retrieved.
    ///
    /// * `frameCtx` - The context passed to the filter's "getFrame" function.
    ///
    /// Returns a pointer to the requested frame, or `NULL` if the requested frame is
    /// not available for any reason. The ownership of the frame is transferred to the caller.
    pub getFrameFilter: unsafe extern "system-unwind" fn(
        n: c_int,
        node: *mut VSNode,
        frameCtx: *mut VSFrameContext,
    ) -> *const VSFrame,
    /// Requests a frame from a node and returns immediately.
    ///
    /// Only use inside a filter's "getFrame" function.
    ///
    /// A filter usually calls this function when its activation reason is arInitial.
    /// The requested frame can then be retrieved using
    /// [`getFrameFilter()`](Self::getFrameFilter), when the filter's activation reason is
    /// [`VSActivationReason::AllFramesReady`].
    ///
    /// It is best to request frames in ascending order, i.e. n, n+1, n+2, etc.
    ///
    /// # Arguments
    ///
    /// * `n` - The frame number. Negative values will cause an error.
    ///
    /// * `node` - The node from which the frame is requested.
    ///
    /// * `frameCtx` - The context passed to the filter's "getFrame" function.
    pub requestFrameFilter: unsafe extern "system-unwind" fn(
        n: c_int,
        node: *mut VSNode,
        frameCtx: *mut VSFrameContext,
    ),
    /// By default all requested frames are referenced until a filter's frame request is done.
    /// In extreme cases where a filter needs to reduce 20+ frames into a single output frame
    /// it may be beneficial to request these in batches
    /// and incrementally process the data instead.
    ///
    /// Should rarely be needed.
    ///
    /// Only use inside a filter's "getFrame" function.
    ///
    /// # Arguments
    ///
    /// * `n` - The frame number. Negative values will cause an error.
    ///
    /// * `node` - The node from which the frame is requested.
    ///
    /// * `frameCtx` - The context passed to the filter's "getFrame" function.
    pub releaseFrameEarly: unsafe extern "system-unwind" fn(
        node: *mut VSNode,
        n: c_int,
        frameCtx: *mut VSFrameContext,
    ),
    /// Pushes a not requested frame into the cache. This is useful for (source) filters
    /// that greatly benefit from completely linear access
    /// and producing all output in linear order.
    ///
    /// This function may only be used in filters that were created with
    /// [`setLinearFilter`](Self::setLinearFilter).
    ///
    /// Only use inside a filter's "getFrame" function.
    pub cacheFrame: unsafe extern "system-unwind" fn(
        frame: *const VSFrame,
        n: c_int,
        frameCtx: *mut VSFrameContext,
    ),
    /// Adds an error message to a frame context, replacing the existing message, if any.
    ///
    /// This is the way to report errors in a filter's "getFrame" function.
    /// Such errors are not necessarily fatal, i.e. the caller can try to
    /// request the same frame again.
    pub setFilterError: unsafe extern "system-unwind" fn(
        errorMessage: *const c_char,
        frameCtx: *mut VSFrameContext,
    ),
    // !SECTION

    // SECTION - External functions
    /// # Arguments
    ///
    /// * `func` - User-defined function that may be called in any context.
    ///
    /// * `userData` - Pointer passed to `func`.
    ///
    /// * `free` - Callback tasked with freeing userData. Can be `NULL`.
    pub createFunction: unsafe extern "system-unwind" fn(
        func: VSPublicFunction,
        userData: *mut c_void,
        free: VSFreeFunctionData,
        core: *mut VSCore,
    ) -> *mut VSFunction,
    /// Decrements the reference count of a function and deletes it when it reaches 0.
    ///
    /// It is safe to pass `NULL`.
    pub freeFunction: unsafe extern "system-unwind" fn(f: *mut VSFunction),
    /// Increments the reference count of a function. Returns f as a convenience.
    pub addFunctionRef: unsafe extern "system-unwind" fn(f: *mut VSFunction) -> *mut VSFunction,
    /// Calls a function. If the call fails out will have an error set.
    ///
    /// # Arguments
    ///
    /// * `func` - Function to be called.
    ///
    /// * `in_` - Arguments passed to `func`.
    ///
    /// * `out` - Returned values from `func`.
    pub callFunction:
        unsafe extern "system-unwind" fn(func: *mut VSFunction, in_: *const VSMap, out: *mut VSMap),
    // !SECTION

    // SECTION - Map and property access functions
    /// Creates a new property map. It must be deallocated later with [`freeMap()`](Self::freeMap).
    pub createMap: unsafe extern "system-unwind" fn() -> *mut VSMap,
    /// Frees a map and all the objects it contains.
    pub freeMap: unsafe extern "system-unwind" fn(map: *mut VSMap),
    /// Deletes all the keys and their associated values from the map, leaving it empty.
    pub clearMap: unsafe extern "system-unwind" fn(map: *mut VSMap),
    /// copies all values in src to dst, if a key already exists in dst it's replaced
    pub copyMap: unsafe extern "system-unwind" fn(src: *const VSMap, dst: *mut VSMap),

    /// Adds an error message to a map. The map is cleared first.
    /// The error message is copied. In this state the map may only be freed,
    /// cleared or queried for the error message.
    ///
    /// For errors encountered in a filter's "getFrame" function, use
    /// [`setFilterError()`](Self::setFilterError).
    pub mapSetError: unsafe extern "system-unwind" fn(map: *mut VSMap, errorMessage: *const c_char),
    /// Returns a pointer to the error message contained in the map,
    /// or `NULL` if there is no error set. The pointer is valid until
    /// the next modifying operation on the map.
    pub mapGetError: unsafe extern "system-unwind" fn(map: *const VSMap) -> *const c_char,

    /// Returns the number of keys contained in a property map.
    pub mapNumKeys: unsafe extern "system-unwind" fn(map: *const VSMap) -> c_int,
    /// Returns the nth key from a property map.
    ///
    /// Passing an invalid index will cause a fatal error.
    ///
    /// The pointer is valid as long as the key exists in the map.
    pub mapGetKey:
        unsafe extern "system-unwind" fn(map: *const VSMap, index: c_int) -> *const c_char,
    /// Removes the property with the given key. All values associated with the key are lost.
    ///
    /// Returns 0 if the key isn't in the map. Otherwise it returns 1.
    pub mapDeleteKey:
        unsafe extern "system-unwind" fn(map: *mut VSMap, key: *const c_char) -> c_int,
    /// Returns the number of elements associated with a key in a property map.
    ///
    /// Returns -1 if there is no such key in the map.
    pub mapNumElements:
        unsafe extern "system-unwind" fn(map: *const VSMap, key: *const c_char) -> c_int,
    /// Returns a value from [`VSPropertyType`] representing type of elements in the given key.
    /// If there is no such key in the map, the returned value is
    /// [`VSPropertyType::Unset`]).
    /// Note that also empty arrays created with mapSetEmpty are typed.
    pub mapGetType:
        unsafe extern "system-unwind" fn(map: *const VSMap, key: *const c_char) -> VSPropertyType,
    /// Creates an empty array of type in key.
    ///
    /// Returns non-zero value on failure due to key already existing or having an invalid name.
    pub mapSetEmpty: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        type_: VSPropertyType,
    ) -> c_int,

    /// Retrieves an integer from a specified key in a map.
    ///
    /// Returns the number on success, or 0 in case of error.
    ///
    /// If the map has an error set (i.e. if [`mapGetError()`](VSAPI::mapGetError)
    /// returns non-`NULL`), `VapourSynth` will die with a fatal error.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index of the element.
    ///
    ///     Use [`mapNumElements()`](Self::mapNumElements) to know the total number of elements
    ///     associated with a key.
    ///
    /// * `error` - One of [`VSMapPropertyError`], [`VSMapPropertyError::Success`]
    ///     on success.
    ///
    ///     You may pass `NULL` here, but then any problems encountered while retrieving
    ///     the property will cause `VapourSynth` to die with a fatal error.
    pub mapGetInt: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut VSMapPropertyError,
    ) -> i64,
    /// Works just like [`mapGetInt()`](Self::mapGetInt) except that the value returned is
    /// also converted to an integer using saturation.
    pub mapGetIntSaturated: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut VSMapPropertyError,
    ) -> c_int,
    /// Retrieves an array of integers from a map. Use this function if there are a lot of numbers
    /// associated with a key, because it is faster than calling
    /// [`mapGetInt()`](Self::mapGetInt) in a loop.
    ///
    /// Returns a pointer to the first element of the array on success,
    /// or `NULL` in case of error. Use [`mapNumElements()`](Self::mapNumElements) to
    /// know the total number of elements associated with a key.
    ///
    /// See [`mapGetInt()`](Self::mapGetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapGetIntArray: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        error: *mut VSMapPropertyError,
    ) -> *const i64,
    /// Sets an integer to the specified key in a map.
    ///
    /// Multiple values can be associated with one key, but they must all be the same type.
    ///
    /// # Arguments
    ///
    /// * `key` - Name of the property. Alphanumeric characters and underscore may be used.
    ///
    /// * `i` - Value to store.
    ///
    /// * `append` - One of [`VSMapAppendMode`].
    ///
    /// Returns 0 on success, or 1 if trying to append to
    /// a property with the wrong type to an existing key.
    pub mapSetInt: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        i: i64,
        append: VSMapAppendMode,
    ) -> c_int,
    /// Adds an array of integers to a map. Use this function if there are a lot of numbers
    /// to add because it is faster than calling [`mapSetInt()`](Self::mapSetInt) in a loop.
    ///
    /// If map already contains a property with this key, that property will be overwritten and
    /// all old values will be lost.
    ///
    /// # Arguments
    ///
    /// * `key` - Name of the property. Alphanumeric characters and underscore may be used.
    ///
    /// * `i` - Pointer to the first element of the array to store.
    ///
    /// * `size` - Number of integers to read from the array. It can be 0, in which case
    ///     no integers are read from the array, and the property will be created empty.
    ///
    /// Returns 0 on success, or 1 if size is negative.
    pub mapSetIntArray: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        i: *const i64,
        size: c_int,
    ) -> c_int,

    /// Retrieves a floating point number from a map.
    ///
    /// Returns the number on success, or 0 in case of error.
    ///
    /// See [`mapGetInt()`](Self::mapGetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapGetFloat: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut VSMapPropertyError,
    ) -> c_double,
    /// Works just like [`mapGetFloat()`](Self::mapGetFloat) except that the value returned
    /// is also converted to a float.
    pub mapGetFloatSaturated: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut VSMapPropertyError,
    ) -> c_float,
    /// Retrieves an array of floating point numbers from a map. Use this function if there are
    /// a lot of numbers associated with a key, because it is faster than calling
    /// [`mapGetFloat()`](Self::mapGetFloat) in a loop.
    ///
    /// Returns a pointer to the first element of the array on success,
    /// or `NULL` in case of error. Use [`mapNumElements()`](Self::mapNumElements) to
    /// know the total number of elements associated with a key.
    ///
    /// See [`mapGetInt()`](Self::mapGetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapGetFloatArray: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        error: *mut VSMapPropertyError,
    ) -> *const c_double,
    /// Sets a float to the specified key in a map.
    ///
    /// See [`mapSetInt()`](Self::mapSetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapSetFloat: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        d: c_double,
        append: VSMapAppendMode,
    ) -> c_int,
    /// Adds an array of floating point numbers to a map. Use this function if there are
    /// a lot of numbers to add, because it is faster than calling
    /// [`mapSetFloat()`](Self::mapSetFloat) in a loop.
    ///
    /// If map already contains a property with this key, that property will be overwritten and
    /// all old values will be lost.
    ///
    /// # Arguments
    ///
    /// * `key` - Name of the property. Alphanumeric characters and underscore may be used.
    ///
    /// * `d` - Pointer to the first element of the array to store.
    ///
    /// * `size` - Number of floating point numbers to read from the array. It can be 0,
    ///     in which case no numbers are read from the array,
    ///     and the property will be created empty.
    ///
    /// Returns 0 on success, or 1 if size is negative.
    pub mapSetFloatArray: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        d: *const c_double,
        size: c_int,
    ) -> c_int,

    /// Retrieves arbitrary binary data from a map. Checking
    /// [`mapGetDataTypeHint()`](Self::mapGetDataTypeHint) may provide a hint about
    /// whether or not the data is human readable.
    ///
    /// Returns a pointer to the data on success, or `NULL` in case of error.
    ///
    /// The array returned is guaranteed to be `NULL`-terminated.
    /// The `NULL` byte is not considered to be part of the array
    /// ([`mapGetDataSize`](Self::mapGetDataSize) doesn't count it).
    ///
    /// The pointer is valid until the map is destroyed, or until the corresponding key
    /// is removed from the map or altered.
    ///
    /// See [`mapGetInt()`](Self::mapGetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapGetData: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut VSMapPropertyError,
    ) -> *const c_char,
    /// Returns the size in bytes of a property of type ptData (see [`VSPropertyType`]),
    /// or 0 in case of error. The terminating `NULL` byte added by
    /// [`mapSetData()`](Self::mapSetData) is not counted.
    ///
    /// See [`mapGetInt()`](Self::mapGetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapGetDataSize: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut VSMapPropertyError,
    ) -> c_int,
    /// Returns the size in bytes of a property of type ptData (see [`VSPropertyType`]),
    /// or 0 in case of error. The terminating `NULL` byte added by
    /// [`mapSetData()`](Self::mapSetData) is not counted.
    ///
    /// See [`mapGetInt()`](Self::mapGetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapGetDataTypeHint: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut VSMapPropertyError,
    ) -> VSDataTypeHint,
    /// Sets binary data to the specified key in a map.
    ///
    /// Multiple values can be associated with one key, but they must all be the same type.
    ///
    /// # Arguments
    ///
    /// * `key` - Name of the property. Alphanumeric characters and the underscore may be used.
    ///
    /// * `data` - Value to store.
    ///
    ///     This function copies the data, so the pointer should be freed when no longer needed.
    ///     A terminating `NULL` is always added to the copied data but not included in
    ///     the total size to make string handling easier.
    ///
    /// * `size` - The number of bytes to copy. If this is negative,
    ///     everything up to the first `NULL` byte will be copied.
    ///
    /// * `type` - One of [`VSDataTypeHint`] to hint whether or not it is human readable data.
    ///
    /// * `append` - One of [`VSMapAppendMode`].
    ///
    /// Returns 0 on success, or 1 if trying to append to a property with the wrong type.
    pub mapSetData: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        data: *const c_char,
        size: c_int,
        type_: VSDataTypeHint,
        append: VSMapAppendMode,
    ) -> c_int,

    /// Retrieves a node from a map.
    ///
    /// Returns a pointer to the node on success, or `NULL` in case of error.
    ///
    /// This function increases the node's reference count, so [`freeNode()`](Self::freeNode)
    /// must be used when the node is no longer needed.
    ///
    /// See [`mapGetInt()`](Self::mapGetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapGetNode: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut VSMapPropertyError,
    ) -> *mut VSNode,
    /// Sets a node to the specified key in a map.
    ///
    /// See [`mapSetInt()`](Self::mapSetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapSetNode: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        node: *mut VSNode,
        append: VSMapAppendMode,
    ) -> c_int,
    /// Sets a node to the specified key in a map and decreases the reference count.
    ///
    /// See [`mapSetInt()`](Self::mapSetInt) for a complete description of
    /// the arguments and general behavior.
    ///
    /// Note: always consumes the reference, even on error
    pub mapConsumeNode: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        node: *mut VSNode,
        append: VSMapAppendMode,
    ) -> c_int,

    /// Retrieves a frame from a map.
    ///
    /// Returns a pointer to the frame on success, or `NULL` in case of error.
    ///
    /// This function increases the frame's reference count, so
    /// [`freeFrame()`](Self::freeFrame) must be used when the frame is no longer needed.
    ///
    /// See [`mapGetInt()`](Self::mapGetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapGetFrame: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut VSMapPropertyError,
    ) -> *const VSFrame,
    /// Sets a frame to the specified key in a map.
    ///
    /// See [`mapSetInt()`](Self::mapSetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapSetFrame: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        f: *const VSFrame,
        append: VSMapAppendMode,
    ) -> c_int,
    /// Sets a frame to the specified key in a map and decreases the reference count.
    ///
    /// See [`mapSetInt()`](Self::mapSetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapConsumeFrame: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        f: *const VSFrame,
        append: VSMapAppendMode,
    ) -> c_int,

    /// Retrieves a function from a map.
    ///
    /// Returns a pointer to the function on success, or `NULL` in case of error.
    ///
    /// This function increases the function's reference count, so
    /// [`freeFunction()`](Self::freeFunction) must be used when the function is no longer needed.
    ///
    /// See [`mapGetInt()`](Self::mapGetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapGetFunction: unsafe extern "system-unwind" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut VSMapPropertyError,
    ) -> *mut VSFunction,
    /// Sets a function object to the specified key in a map.
    ///
    /// See [`mapSetInt()`](Self::mapSetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapSetFunction: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        func: *mut VSFunction,
        append: VSMapAppendMode,
    ) -> c_int,
    /// Sets a function object to the specified key in a map and decreases the reference count.
    ///
    /// See [`mapSetInt()`](Self::mapSetInt) for a complete description of
    /// the arguments and general behavior.
    pub mapConsumeFunction: unsafe extern "system-unwind" fn(
        map: *mut VSMap,
        key: *const c_char,
        func: *mut VSFunction,
        append: VSMapAppendMode,
    ) -> c_int,
    // !SECTION

    // SECTION - Plugin and plugin function related
    /// Function that registers a filter exported by the plugin.
    /// A plugin can export any number of filters. This function may only be called during
    /// the plugin loading phase unless the [`VSPluginConfigFlags::Modifiable`] flag was
    /// set by [`configPlugin`](VSPLUGINAPI::configPlugin).
    ///
    /// # Arguments
    ///
    /// * `name` - Filter name. The characters allowed are letters, numbers, and the underscore.
    ///     The first character must be a letter. In other words: ^[a-zA-Z][a-zA-Z0-9_]*$
    ///
    ///     Filter names _should be_ `PascalCase`.
    ///
    /// * `args` - String containing the filter's list of arguments.
    ///
    ///     Arguments are separated by a semicolon. Each argument is made of several fields
    ///     separated by a colon. Don't insert additional whitespace characters,
    ///     or `VapourSynth` will die.
    ///
    ///     ## Fields:
    ///
    ///     * The argument name.
    ///
    ///         The same characters are allowed as for the filter's name.
    ///         Argument names should be all lowercase and use only letters and the underscore.
    ///
    ///     * The type.
    ///
    ///         * "int": `int64_t`
    ///         * "float": double
    ///         * "data": const char*
    ///         * "anode": const [`VSNode`]* (audio type)
    ///         * "vnode": const [`VSNode`]* (video type)
    ///         * "aframe": const [`VSFrame`]* (audio type)
    ///         * "vframe": const [`VSFrame`]* (video type)
    ///         * "func": const [`VSFunction`]*
    ///
    ///         It is possible to declare an array by appending "[]" to the type.
    ///
    ///     * "opt"
    ///
    ///         If the parameter is optional.
    ///
    ///     * "empty"
    ///
    ///         For arrays that are allowed to be empty.
    ///
    ///     * "any"
    ///
    ///         Can only be placed last without a semicolon after.
    ///         Indicates that all remaining arguments that don't match
    ///         should also be passed through.
    ///
    ///     ## Example
    ///
    ///     The following example declares the arguments "blah", "moo", and "asdf":
    ///
    ///     ```txt
    ///     blah:vnode;moo:int[]:opt;asdf:float:opt;
    ///     ```
    ///
    ///     The following example declares the arguments "blah" and accepts all other arguments
    ///     no matter the type:
    ///
    ///     ```txt
    ///     blah:vnode;any
    ///     ```
    ///
    /// * `returnType` - Specifies works similarly to `args` but instead specifies which keys
    ///     and what type will be returned. Typically this will be:
    ///
    ///     ```txt
    ///     clip:vnode;
    ///     ```
    ///
    ///     for video filters. It is important to not simply specify "any" for all filters
    ///     since this information is used for better auto-completion in many editors.
    ///
    /// * `argsFunc` -  See [`VSPublicFunction`].
    ///
    /// * `functionData` - Pointer to user data that gets passed to `argsFunc`
    ///     when creating a filter. Useful to register multiple filters using
    ///     a single `argsFunc` function.
    ///
    /// * `plugin` - Pointer to the plugin object in the core, as passed to
    ///     `VapourSynthPluginInit2()`.
    pub registerFunction: unsafe extern "system-unwind" fn(
        name: *const c_char,
        args: *const c_char,
        returnType: *const c_char,
        argsFunc: VSPublicFunction,
        functionData: *mut c_void,
        plugin: *mut VSPlugin,
    ) -> c_int,
    /// Returns a pointer to the plugin with the given identifier, or NULL if not found.
    ///
    /// # Arguments
    ///
    /// * `identifier` - Reverse URL that uniquely identifies the plugin.
    pub getPluginByID: unsafe extern "system-unwind" fn(
        identifier: *const c_char,
        core: *mut VSCore,
    ) -> *mut VSPlugin,
    /// Returns a pointer to the plugin with the given namespace, or `NULL` if not found.
    ///
    /// [`getPluginByID`](Self::getPluginByID) is generally a better option.
    ///
    /// # Arguments
    ///
    /// * `ns` - Namespace.
    pub getPluginByNamespace:
        unsafe extern "system-unwind" fn(ns: *const c_char, core: *mut VSCore) -> *mut VSPlugin,
    /// Used to enumerate over all currently loaded plugins.
    /// The order is fixed but provides no other guarantees.
    ///
    /// # Arguments
    ///
    /// * `plugin` - Current plugin. Pass `NULL` to get the first plugin.
    ///
    /// Returns a pointer to the next plugin in order or
    /// `NULL` if the final plugin has been reached.
    pub getNextPlugin:
        unsafe extern "system-unwind" fn(plugin: *mut VSPlugin, core: *mut VSCore) -> *mut VSPlugin,
    /// Returns the name of the plugin that was passed to
    /// [`configPlugin`](VSPLUGINAPI::configPlugin).
    pub getPluginName: unsafe extern "system-unwind" fn(plugin: *mut VSPlugin) -> *const c_char,
    /// Returns the identifier of the plugin that was passed to
    /// [`configPlugin`](VSPLUGINAPI::configPlugin).
    pub getPluginID: unsafe extern "system-unwind" fn(plugin: *mut VSPlugin) -> *const c_char,
    /// Returns the namespace the plugin currently is loaded in.
    pub getPluginNamespace:
        unsafe extern "system-unwind" fn(plugin: *mut VSPlugin) -> *const c_char,
    /// Used to enumerate over all functions in a plugin.
    /// The order is fixed but provides no other guarantees.
    ///
    /// # Arguments
    ///
    /// * `func` - Current function. Pass `NULL` to get the first function.
    ///
    /// * `plugin` - The plugin to enumerate functions in.
    ///
    /// Returns a pointer to the next function in order or
    /// `NULL` if the final function has been reached.
    pub getNextPluginFunction: unsafe extern "system-unwind" fn(
        func: *mut VSPluginFunction,
        plugin: *mut VSPlugin,
    ) -> *mut VSPluginFunction,
    /// Get a function belonging to a plugin by its name.
    pub getPluginFunctionByName: unsafe extern "system-unwind" fn(
        name: *const c_char,
        plugin: *mut VSPlugin,
    ) -> *mut VSPluginFunction,
    /// Returns the name of the function that was passed to
    /// [`registerFunction()`](Self::registerFunction).
    pub getPluginFunctionName:
        unsafe extern "system-unwind" fn(func: *mut VSPluginFunction) -> *const c_char,
    /// Returns the argument string of the function that was passed to
    /// [`registerFunction()`](Self::registerFunction).
    pub getPluginFunctionArguments:
        unsafe extern "system-unwind" fn(func: *mut VSPluginFunction) -> *const c_char,
    /// Returns the return type string of the function that was passed to
    /// [`registerFunction()`](Self::registerFunction).
    pub getPluginFunctionReturnType:
        unsafe extern "system-unwind" fn(func: *mut VSPluginFunction) -> *const c_char,
    /// Returns the absolute path to the plugin, including the plugin's file name.
    /// This is the real location of the plugin, i.e. there are no symbolic links in the path.
    ///
    /// Path elements are always delimited with forward slashes.
    ///
    /// `VapourSynth` retains ownership of the returned pointer.
    pub getPluginPath: unsafe extern "system-unwind" fn(plugin: *const VSPlugin) -> *const c_char,
    /// Returns the version of the plugin.
    /// This is the same as the version number passed to
    /// [`configPlugin()`](VSPLUGINAPI::configPlugin).
    pub getPluginVersion: unsafe extern "system-unwind" fn(plugin: *const VSPlugin) -> c_int,
    /// Invokes a filter.
    ///
    /// [`invoke()`](Self::invoke) checks that the args passed to the filter are consistent
    /// with the argument list registered by the plugin that contains the filter,
    /// calls the filter's "create" function, and checks that
    /// the filter returns the declared types.
    /// If everything goes smoothly, the filter will be ready to generate frames after
    /// [`invoke()`](Self::invoke) returns.
    ///
    /// # Arguments
    ///
    /// * `plugin` - A pointer to the plugin where the filter is located. Must not be `NULL`.
    ///
    ///     See [`getPluginByID()`](Self::getPluginByID).
    ///
    /// * `name` - Name of the filter to invoke.
    ///
    /// * `args` - Arguments for the filter.
    ///
    /// Returns a map containing the filter's return value(s).
    /// The caller takes ownership of the map.
    /// Use [`mapGetError()`](Self::mapGetError) to check if the filter was invoked successfully.
    ///
    /// Most filters will either set an error, or one or more clips with the key "clip".
    /// The exception to this are functions, for example `LoadPlugin`,
    /// which doesn't return any clips for obvious reasons.
    pub invoke: unsafe extern "system-unwind" fn(
        plugin: *mut VSPlugin,
        name: *const c_char,
        args: *const VSMap,
    ) -> *mut VSMap,
    // !SECTION

    // SECTION - Core and information
    /// Creates the `VapourSynth` processing core and returns a pointer to it.
    /// It is possible to create multiple cores but in most cases it shouldn't be needed.
    ///
    /// # Arguments
    ///
    /// * `flags` - [`VSCoreCreationFlags`] `ORed` together if desired.
    ///    Pass 0 for sane defaults that should suit most uses.
    ///
    pub createCore: unsafe extern "system-unwind" fn(flags: c_int) -> *mut VSCore,

    /// Frees a core. Should only be done after all frame requests have completed
    /// and all objects belonging to the core have been released.
    pub freeCore: unsafe extern "system-unwind" fn(core: *mut VSCore),

    /// Sets the maximum size of the framebuffer cache.
    ///
    /// Note: the total cache size at which vapoursynth more aggressively tries to reclaim memory,
    /// it is not a hard limit
    ///
    /// # Return:
    ///
    /// the new maximum size.
    pub setMaxCacheSize: unsafe extern "system-unwind" fn(bytes: i64, core: *mut VSCore) -> i64,

    /// Sets the number of threads used for processing. Pass 0 to automatically detect.
    /// Returns the number of threads that will be used for processing.
    pub setThreadCount:
        unsafe extern "system-unwind" fn(threads: c_int, core: *mut VSCore) -> c_int,

    /// Returns information about the `VapourSynth` core.
    pub getCoreInfo: unsafe extern "system-unwind" fn(core: *mut VSCore, info: *mut VSCoreInfo),

    /// Returns the highest [`VAPOURSYNTH_API_VERSION`]
    /// the library support.
    pub getAPIVersion: unsafe extern "system-unwind" fn() -> c_int,
    // !SECTION

    // SECTION - Message handler
    /// Send a message through `VapourSynth`'s logging framework.
    /// See [`addLogHandler`](Self::addLogHandler).
    ///
    /// # Arguments
    /// * `msgType` - The type of message. One of [`VSMessageType`].
    ///
    ///     If `msgType` is [`VSMessageType::Fatal`],
    ///     `VapourSynth` will call `abort()` after delivering the message.
    ///
    /// * `msg` - The message.
    pub logMessage: unsafe extern "system-unwind" fn(
        msgType: VSMessageType,
        msg: *const c_char,
        core: *mut VSCore,
    ),
    /// Installs a custom handler for the various error messages `VapourSynth` emits.
    /// The message handler is per [`VSCore`] instance. Returns a unique handle.
    ///
    /// If no log handler is installed up to a few hundred messages are cached and
    /// will be delivered as soon as a log handler is attached. This behavior exists mostly
    /// so that warnings when auto-loading plugins (default behavior) won't disappear-
    ///
    /// # Arguments
    ///
    /// * `handler` -  Custom message handler. If this is `NULL`,
    ///     the default message handler will be restored.
    ///
    /// * `free` - Called when a handler is removed.
    ///
    /// * `userData` - Pointer that gets passed to the message handler.
    pub addLogHandler: unsafe extern "system-unwind" fn(
        handler: VSLogHandler,
        free: VSLogHandlerFree,
        userData: *mut c_void,
        core: *mut VSCore,
    ) -> *mut VSLogHandle,
    /// Removes a custom handler. Return non-zero on success and zero if the handle is invalid.
    ///
    /// # Arguments
    ///
    /// * `handle` - Handle obtained from [`addLogHandler()`](Self::addLogHandler).
    pub removeLogHandler:
        unsafe extern "system-unwind" fn(handle: *mut VSLogHandle, core: *mut VSCore) -> c_int,

    // !SECTION

    // MARK: API 4.1
    // mostly graph and node inspection, PLEASE DON'T USE INSIDE FILTERS

    /* Additional cache management to free memory */
    /// clears the cache of the specified node
    #[cfg(feature = "vs-41")]
    pub clearNodeCache: unsafe extern "system-unwind" fn(node: *mut VSNode),
    /// clears all caches belonging to the specified core
    #[cfg(feature = "vs-41")]
    pub clearCoreCaches: unsafe extern "system-unwind" fn(core: *mut VSCore),

    /* Basic node information */
    /// the name passed to `create*Filter*`
    #[cfg(feature = "vs-41")]
    pub getNodeName: unsafe extern "system-unwind" fn(node: *mut VSNode) -> *const c_char,
    #[cfg(feature = "vs-41")]
    /// returns [`VSFilterMode`]
    pub getNodeFilterMode: unsafe extern "system-unwind" fn(node: *mut VSNode) -> VSFilterMode,
    #[cfg(feature = "vs-41")]
    pub getNumNodeDependencies: unsafe extern "system-unwind" fn(node: *mut VSNode) -> c_int,
    #[cfg(feature = "vs-41")]
    pub getNodeDependencies:
        unsafe extern "system-unwind" fn(node: *mut VSNode) -> *const VSFilterDependency,

    /* Node timing functions */
    /// non-zero when filter timing is enabled
    pub getCoreNodeTiming: unsafe extern "system-unwind" fn(core: *mut VSCore) -> c_int,
    /// non-zero enables filter timing, note that disabling simply stops the counters from incrementing
    pub setCoreNodeTiming: unsafe extern "system-unwind" fn(core: *mut VSCore, enable: c_int),
    /// time spent processing frames in nanoseconds, reset sets the counter to 0 again
    pub getNodeProcessingTime:
        unsafe extern "system-unwind" fn(node: *mut VSNode, reset: c_int) -> i64,
    /// time spent processing frames in nanoseconds in all destroyed nodes, reset sets the counter to 0 again
    pub getFreedNodeProcessingTime:
        unsafe extern "system-unwind" fn(core: *mut VSCore, reset: c_int) -> i64,

    // MARK: Graph information
    /*
     * !!! Experimental/expensive graph information
     * These functions only exist to retrieve internal details for debug purposes and
     * graph visualization They will only only work properly when used on a core created
     * with `ccfEnableGraphInspection` and are not safe to use concurrently with frame requests
     * or other API functions. Because of this they are unsuitable for use in plugins and filters.
     */
    /// level=0 returns the name of the function that created the filter,
    /// specifying a higher level will retrieve the function above that
    /// invoked it or `NULL` if a non-existent level is requested
    #[cfg(feature = "vs-graph")]
    pub getNodeCreationFunctionName:
        unsafe extern "system-unwind" fn(node: *mut VSNode, level: c_int) -> *const c_char,
    /// level=0 returns a copy of the arguments passed to the function that created the filter,
    /// returns `NULL` if a non-existent level is requested
    #[cfg(feature = "vs-graph")]
    pub getNodeCreationFunctionArguments:
        unsafe extern "system-unwind" fn(node: *mut VSNode, level: c_int) -> *const VSMap,
}

extern "system-unwind" {
    /// Returns a pointer to the global [`VSAPI`] instance.
    ///
    /// Returns `NULL` if the requested API version is not supported or
    /// if the system does not meet the minimum requirements to run `VapourSynth`.
    /// It is recommended to pass [`VAPOURSYNTH_API_VERSION`]
    pub fn getVapourSynthAPI(version: c_int) -> *const VSAPI;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout() {
        assert_eq!(
            std::mem::size_of::<VSPresetVideoFormat>(),
            std::mem::size_of::<c_int>(),
            "VSPresetFormat"
        );
        assert_eq!(
            std::mem::size_of::<VSDataTypeHint>(),
            std::mem::size_of::<c_int>(),
            "VSDataTypeHint"
        );
        assert_eq!(
            std::mem::size_of::<VSCoreCreationFlags>(),
            std::mem::size_of::<c_int>(),
            "VSCoreCreationFlags"
        );
        assert_eq!(
            std::mem::size_of::<VSFilterMode>(),
            std::mem::size_of::<c_int>(),
            "VSFilterMode"
        );
        assert_eq!(
            std::mem::size_of::<VSColorFamily>(),
            std::mem::size_of::<c_int>(),
            "VSColorFamily"
        );
        assert_eq!(
            std::mem::size_of::<VSSampleType>(),
            std::mem::size_of::<c_int>(),
            "VSSampleType"
        );
        assert_eq!(
            std::mem::size_of::<VSMapAppendMode>(),
            std::mem::size_of::<c_int>(),
            "VSMapAppendMode"
        );
        assert_eq!(
            std::mem::size_of::<VSMessageType>(),
            std::mem::size_of::<c_int>(),
            "VSMessageType"
        );
        assert_eq!(
            std::mem::size_of::<VSCacheMode>(),
            std::mem::size_of::<c_int>(),
            "VSCacheMode"
        );
    }
}

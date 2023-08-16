/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

//! VapourSynth4.h

use core::ffi::*;

pub const VS_AUDIO_FRAME_SAMPLES: i32 = 3072;

use super::opaque_struct;

opaque_struct!(
    VSFrame,
    VSNode,
    VSCore,
    VSPlugin,
    VSPluginFunction,
    VSFunction,
    VSMap,
    VSLogHandle,
    VSFrameContext
);

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSColorFamily {
    cfUndefined = 0,
    cfGray = 1,
    cfRGB = 2,
    cfYUV = 3,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSSampleType {
    stInteger = 0,
    stFloat = 1,
}

const fn VS_MAKE_VIDEO_ID(
    colorFamily: VSColorFamily,
    sampleType: VSSampleType,
    bitsPerSample: isize,
    subSamplingW: isize,
    subSamplingH: isize,
) -> isize {
    ((colorFamily as isize) << 28)
        | ((sampleType as isize) << 24)
        | (bitsPerSample << 16)
        | (subSamplingW << 8)
        | subSamplingH
}

use VSColorFamily::*;
use VSSampleType::*;

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSPresetFormat {
    pfNone = 0,

    pfGray8 = VS_MAKE_VIDEO_ID(cfGray, stInteger, 8, 0, 0),
    pfGray9 = VS_MAKE_VIDEO_ID(cfGray, stInteger, 9, 0, 0),
    pfGray10 = VS_MAKE_VIDEO_ID(cfGray, stInteger, 10, 0, 0),
    pfGray12 = VS_MAKE_VIDEO_ID(cfGray, stInteger, 12, 0, 0),
    pfGray14 = VS_MAKE_VIDEO_ID(cfGray, stInteger, 14, 0, 0),
    pfGray16 = VS_MAKE_VIDEO_ID(cfGray, stInteger, 16, 0, 0),
    pfGray32 = VS_MAKE_VIDEO_ID(cfGray, stInteger, 32, 0, 0),

    pfGrayH = VS_MAKE_VIDEO_ID(cfGray, stFloat, 16, 0, 0),
    pfGrayS = VS_MAKE_VIDEO_ID(cfGray, stFloat, 32, 0, 0),

    pfYUV410P8 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 8, 2, 2),
    pfYUV411P8 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 8, 2, 0),
    pfYUV440P8 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 8, 0, 1),

    pfYUV420P8 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 8, 1, 1),
    pfYUV422P8 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 8, 1, 0),
    pfYUV444P8 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 8, 0, 0),

    pfYUV420P9 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 9, 1, 1),
    pfYUV422P9 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 9, 1, 0),
    pfYUV444P9 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 9, 0, 0),

    pfYUV420P10 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 10, 1, 1),
    pfYUV422P10 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 10, 1, 0),
    pfYUV444P10 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 10, 0, 0),

    pfYUV420P12 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 12, 1, 1),
    pfYUV422P12 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 12, 1, 0),
    pfYUV444P12 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 12, 0, 0),

    pfYUV420P14 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 14, 1, 1),
    pfYUV422P14 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 14, 1, 0),
    pfYUV444P14 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 14, 0, 0),

    pfYUV420P16 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 16, 1, 1),
    pfYUV422P16 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 16, 1, 0),
    pfYUV444P16 = VS_MAKE_VIDEO_ID(cfYUV, stInteger, 16, 0, 0),

    pfYUV444PH = VS_MAKE_VIDEO_ID(cfYUV, stFloat, 16, 0, 0),
    pfYUV444PS = VS_MAKE_VIDEO_ID(cfYUV, stFloat, 32, 0, 0),

    pfRGB24 = VS_MAKE_VIDEO_ID(cfRGB, stInteger, 8, 0, 0),
    pfRGB27 = VS_MAKE_VIDEO_ID(cfRGB, stInteger, 9, 0, 0),
    pfRGB30 = VS_MAKE_VIDEO_ID(cfRGB, stInteger, 10, 0, 0),
    pfRGB36 = VS_MAKE_VIDEO_ID(cfRGB, stInteger, 12, 0, 0),
    pfRGB42 = VS_MAKE_VIDEO_ID(cfRGB, stInteger, 14, 0, 0),
    pfRGB48 = VS_MAKE_VIDEO_ID(cfRGB, stInteger, 16, 0, 0),

    pfRGBH = VS_MAKE_VIDEO_ID(cfRGB, stFloat, 16, 0, 0),
    pfRGBS = VS_MAKE_VIDEO_ID(cfRGB, stFloat, 32, 0, 0),
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSFilterMode {
    /// completely parallel execution
    fmParallel = 0,
    /// for filters that are serial in nature but can request
    /// one or more frames they need in advance
    fmParallelRequests = 1,
    /// for filters that modify their internal state every request
    /// like source filters that read a file
    fmUnordered = 2,
    /// _DO NOT USE UNLESS ABSOLUTELY NECESSARY_,
    /// for compatibility with external code that can only keep the
    /// processing state of a single frame at a time
    fmFrameState = 3,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSMediaType {
    mtVideo = 1,
    mtAudio = 2,
}

#[repr(C)]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VSVideoFormat {
    pub colorFamily: VSColorFamily,
    pub sampleType: VSSampleType,
    /// number of significant bits
    pub bitsPerSample: c_int,
    /// actual storage is always in a power of 2 and the smallest possible
    /// that can fit the number of bits used per sample
    pub bytesPerSample: c_int,

    /// log2 subsampling factor, applied to second and third plane
    pub subSamplingW: c_int,
    /// log2 subsampling factor, applied to second and third plane
    pub subSamplingH: c_int,

    /// implicit from colorFamily
    pub numPlanes: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSAudioChannels {
    acFrontLeft = 0,
    acFrontRight = 1,
    acFrontCenter = 2,
    acLowFrequency = 3,
    acBackLeft = 4,
    acBackRight = 5,
    acFrontLeftOFCenter = 6,
    acFrontRightOFCenter = 7,
    acBackCenter = 8,
    acSideLeft = 9,
    acSideRight = 10,
    acTopCenter = 11,
    acTopFrontLeft = 12,
    acTopFrontCenter = 13,
    acTopFrontRight = 14,
    acTopBackLeft = 15,
    acTopBackCenter = 16,
    acTopBackRight = 17,
    acStereoLeft = 29,
    acStereoRight = 30,
    acWideLeft = 31,
    acWideRight = 32,
    acSurroundDirectLeft = 33,
    acSurroundDirectRight = 34,
    acLowFrequency2 = 35,
}

#[repr(C)]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VSAudioFormat {
    pub sampleType: VSSampleType,
    pub bitsPerSample: c_int,
    /// implicit from channelLayout
    pub bytesPerSample: c_int,
    /// implicit from bitsPerSample
    pub numChannels: c_int,
    pub channelLayout: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSPropertyType {
    ptUnset = 0,
    ptInt = 1,
    ptFloat = 2,
    ptData = 3,
    ptFunction = 4,
    ptVideoNode = 5,
    ptAudioNode = 6,
    ptVideoFrame = 7,
    ptAudioFrame = 8,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSMapPropertyError {
    peSuccess = 0,
    /// no key exists
    peUnset = 1,
    /// key exists but not of a compatible type
    peType = 2,
    /// index out of bounds
    peIndex = 4,
    /// map has error state set
    peError = 3,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSMapAppendMode {
    maReplace = 0,
    maAppend = 1,
}

#[repr(C)]
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct VSCoreInfo {
    pub versionString: *const c_char, // TODO: figure out how to clone
    pub core: c_int,
    pub api: c_int,
    pub numThreads: c_int,
    pub maxFramebufferSize: i64,
    pub usedFramebufferSize: i64,
}

#[repr(C)]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VSVideoInfo {
    pub format: VSVideoFormat,
    pub fpsNum: i64,
    pub fpsDen: i64,
    pub width: c_int,
    pub height: c_int,
    pub numFrames: c_int,
}

#[repr(C)]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct VSAudioInfo {
    pub format: VSAudioFormat,
    pub sampleRate: c_int,
    pub numSamples: i64,
    /// the total number of audio frames needed to hold [`numSamples`](Self::numSamples),
    /// implicit from [`numSamples`](Self::numSamples) when calling
    /// [`createAudioFilter()`](VSAPI::createAudioFilter)
    pub numFrames: c_int,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSActivationReason {
    arInitial = 0,
    arAllFramesReady = 1,
    arError = -1,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSMessageType {
    mtDebug = 0,
    mtInformation = 1,
    mtWarning = 2,
    mtCritical = 3,
    /// also terminates the process, should generally not be used by normal filters
    mtFatal = 4,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSCoreCreationFlags {
    ccfEnableGraphInspection = 1,
    ccfDisableAutoLoading = 2,
    ccfDisableLibraryUnloading = 4,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSPluginConfigFlags {
    pcModifiable = 1,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSDataTypeHint {
    dtUnknown = -1,
    dtBinary = 0,
    dtUtf8 = 1,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSRequestPattern {
    /// General pattern
    rpGeneral = 0,
    /// When requesting all output frames from the filter,
    /// no frame will be requested more than once from this input clip,
    /// never requests frames beyond the end of the clip
    rpNoFrameReuse = 1,
    /// Always (and only) requests frame n from input clip when generating output frame n,
    /// never requests frames beyond the end of the clip
    rpStrictSpatial = 2,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum VSCacheMode {
    cmAuto = -1,
    cmForceDisable = 0,
    cmForceEnable = 1,
}

/// Core entry point
pub type VSGetVapourSynthAPI = unsafe extern "system" fn(version: c_int) -> *const VSAPI;

// Plugin, function and filter related
pub type VSPublicFunction = unsafe extern "system" fn(
    in_: *const VSMap,
    out: *mut VSMap,
    userData: *mut c_void,
    core: *mut VSCore,
    vsapi: *const VSAPI,
);
pub type VSInitPlugin =
    unsafe extern "system" fn(plugin: *mut VSPlugin, vspapi: *const VSPLUGINAPI);
pub type VSFreeFunctionData = Option<unsafe extern "system" fn(userData: *mut c_void)>;
pub type VSFilterGetFrame = unsafe extern "system" fn(
    n: c_int,
    activationReason: c_int,
    instanceData: *mut c_void,
    frameData: *mut *mut c_void,
    frameCtx: *mut VSFrameContext,
    core: *mut VSCore,
    vsapi: *const VSAPI,
) -> *const VSFrame;
pub type VSFilterFree =
    unsafe extern "system" fn(instanceData: *mut c_void, core: *mut VSCore, vsapi: *const VSAPI);

// Other
pub type VSFrameDoneCallback = unsafe extern "system" fn(
    userData: *mut c_void,
    f: *const VSFrame,
    n: c_int,
    node: *mut VSNode,
    errorMsg: *const c_char,
);
pub type VSLogHandler =
    unsafe extern "system" fn(msgType: c_int, msg: *const c_char, userData: *mut c_void);
pub type VSLogHandlerFree = unsafe extern "system" fn(userData: *mut c_void);

#[repr(C)]
pub struct VSPLUGINAPI {
    /// returns [`VAPOURSYNTH_API_VERSION`](crate::VAPOURSYNTH_API_VERSION) of the library
    pub getAPIVersion: unsafe extern "system" fn() -> c_int,
    /// use the [`VS_MAKE_VERSION()`](crate::VS_MAKE_VERSION) const function for `pluginVersion`
    pub configPlugin: unsafe extern "system" fn(
        identifier: *const c_char,
        pluginNamespace: *const c_char,
        name: *const c_char,
        pluginVersion: c_int,
        apiVersion: c_int,
        flags: c_int,
        plugin: *mut VSPlugin,
    ) -> c_int,
    /// non-zero return value on success
    pub registerFunction: unsafe extern "system" fn(
        name: *const c_char,
        args: *const c_char,
        returnType: *const c_char,
        argsFunc: VSPublicFunction,
        functionData: *mut c_void,
        plugin: *mut VSPlugin,
    ) -> c_int,
}

#[repr(C)]
#[derive(Eq, PartialEq, Hash, Debug)]
pub struct VSFilterDependency {
    source: *mut VSNode,
    requestPattern: VSRequestPattern,
}

#[repr(C)]
pub struct VSAPI {
    // Audio and video filter related including nodes
    /// output nodes are appended to the clip key in the out map
    pub createVideoFilter: unsafe extern "system" fn(
        out: *mut VSMap,
        name: *const c_char,
        vi: *const VSVideoInfo,
        getFrame: VSFilterGetFrame,
        free: VSFilterFree,
        filterMode: c_int,
        dependencies: *const VSFilterDependency,
        numDeps: c_int,
        instanceData: *mut c_void,
        core: *mut VSCore,
    ),
    /// same as [`createVideoFilter()`](Self::createVideoFilter)
    /// but returns a pointer to the [`VSNode`] directly or `NULL` on failure
    pub createVideoFilter2: unsafe extern "system" fn(
        name: *const c_char,
        vi: *const VSVideoInfo,
        getFrame: VSFilterGetFrame,
        free: VSFilterFree,
        filterMode: c_int,
        dependencies: *const VSFilterDependency,
        numDeps: c_int,
        instanceData: *mut c_void,
        core: *mut VSCore,
    ) -> *mut VSNode,
    /// output nodes are appended to the clip key in the out map
    pub createAudioFilter: unsafe extern "system" fn(
        out: *mut VSMap,
        name: *const c_char,
        ai: *const VSAudioInfo,
        getFrame: VSFilterGetFrame,
        free: VSFilterFree,
        filterMode: c_int,
        dependencies: *const VSFilterDependency,
        numDeps: c_int,
        instanceData: *mut c_void,
        core: *mut VSCore,
    ),
    /// same as [`createAudioFilter()`](Self::createAudioFilter)
    /// but returns a pointer to the [`VSNode`] directly or NULL on failure
    pub createAudioFilter2: unsafe extern "system" fn(
        name: *const c_char,
        ai: *const VSAudioInfo,
        getFrame: VSFilterGetFrame,
        free: VSFilterFree,
        filterMode: c_int,
        dependencies: *const VSFilterDependency,
        numDeps: c_int,
        instanceData: *mut c_void,
        core: *mut VSCore,
    ) -> *mut VSNode,
    /// Use right after `create*Filter*`, sets the correct cache mode for
    /// using the [`cacheFrame`](Self::cacheFrame) API and returns the recommended upper number of
    /// additional frames to cache per request
    pub setLinearFilter: unsafe extern "system" fn(node: *mut VSNode) -> c_int,
    /// [`VSCacheMode`], changing the cache mode also resets all options to their default
    pub setCacheMode: unsafe extern "system" fn(node: *mut VSNode, mode: c_int),
    /// passing -1 means no change
    pub setCacheOptions: unsafe extern "system" fn(
        node: *mut VSNode,
        fixedSize: c_int,
        maxSize: c_int,
        maxHistorySize: c_int,
    ),

    pub freeNode: unsafe extern "system" fn(node: *mut VSNode),
    pub addNodeRef: unsafe extern "system" fn(node: *mut VSNode) -> *mut VSNode,
    pub getNodeType: unsafe extern "system" fn(node: *mut VSNode) -> VSMediaType,
    pub getVideoInfo: unsafe extern "system" fn(node: *mut VSNode) -> *const VSVideoInfo,
    pub getAudioInfo: unsafe extern "system" fn(node: *mut VSNode) -> *const VSAudioInfo,

    // Frame related functions
    pub newVideoFrame: unsafe extern "system" fn(
        format: *const VSVideoFormat,
        width: c_int,
        height: c_int,
        propSrc: *const VSFrame,
        core: *mut VSCore,
    ) -> *mut VSFrame,
    /// same as [`newVideoFrame()`](Self::newVideoFrame) but allows the specified planes
    /// to be effectively copied from the source frames
    pub newVideoFrame2: unsafe extern "system" fn(
        format: *const VSVideoFormat,
        width: c_int,
        height: c_int,
        planeSrc: *const *const VSFrame,
        planes: *const c_int,
        propSrc: *const VSFrame,
        core: *mut VSCore,
    ) -> *mut VSFrame,
    pub newAudioFrame: unsafe extern "system" fn(
        format: *const VSAudioFormat,
        numSamples: c_int,
        propSrc: *const VSFrame,
        core: *mut VSCore,
    ) -> *mut VSFrame,
    /// same as [`newAudioFrame()`](Self::newAudioFrame) but allows the specified channels
    /// to be effectively copied from the source frames
    pub newAudioFrame2: unsafe extern "system" fn(
        format: *const VSAudioFormat,
        numSamples: c_int,
        channelSrc: *const *const VSFrame,
        channels: *const c_int,
        propSrc: *const VSFrame,
        core: *mut VSCore,
    ) -> *mut VSFrame,
    pub freeFrame: unsafe extern "system" fn(f: *const VSFrame),
    pub addFrameRef: unsafe extern "system" fn(f: *const VSFrame) -> *mut VSFrame,
    pub copyFrame: unsafe extern "system" fn(f: *const VSFrame, core: *mut VSCore) -> *mut VSFrame,
    pub getFramePropertiesRO: unsafe extern "system" fn(f: *const VSFrame) -> *const VSMap,
    pub getFramePropertiesRW: unsafe extern "system" fn(f: *mut VSFrame) -> *mut VSMap,

    pub getStride: unsafe extern "system" fn(f: *const VSFrame, plane: c_int) -> isize,
    pub getReadPtr: unsafe extern "system" fn(f: *const VSFrame, plane: c_int) -> *const u8,
    /// calling this function invalidates previously gotten read pointers to the same frame
    pub getWritePtr: unsafe extern "system" fn(f: *mut VSFrame, plane: c_int) -> *mut u8,

    pub getVideoFrameFormat: unsafe extern "system" fn(f: *const VSFrame) -> *const VSVideoFormat,
    pub getAudioFrameFormat: unsafe extern "system" fn(f: *const VSFrame) -> *const VSAudioFormat,
    pub getFrameType: unsafe extern "system" fn(f: *const VSFrame) -> c_int,
    pub getFrameWidth: unsafe extern "system" fn(f: *const VSFrame, plane: c_int) -> c_int,
    pub getFrameHeight: unsafe extern "system" fn(f: *const VSFrame, plane: c_int) -> c_int,
    pub getFrameLength: unsafe extern "system" fn(f: *const VSFrame) -> c_int,

    // General format functions
    /// up to 32 characters including terminating null may be written to the buffer,
    /// non-zero return value on success
    pub getVideoFormatName:
        unsafe extern "system" fn(format: *const VSVideoFormat, buffer: *mut c_char) -> c_int,
    /// up to 32 characters including terminating null may be written to the buffer,
    /// non-zero return value on success
    pub getAudioFormatName:
        unsafe extern "system" fn(format: *const VSAudioFormat, buffer: *mut c_char) -> c_int,
    /// non-zero return value on success
    pub queryVideoFormat: unsafe extern "system" fn(
        format: *mut VSVideoFormat,
        colorFamily: c_int,
        sampleType: c_int,
        bitsPerSample: c_int,
        subSamplingW: c_int,
        subSamplingH: c_int,
        core: *mut VSCore,
    ) -> c_int,
    /// non-zero return value on success
    pub queryAudioFormat: unsafe extern "system" fn(
        format: *mut VSAudioFormat,
        sampleType: c_int,
        bitsPerSample: c_int,
        channelLayout: u64,
        core: *mut VSCore,
    ) -> c_int,
    /// returns 0 on failure
    pub queryVideoFormatID: unsafe extern "system" fn(
        colorFamily: VSColorFamily,
        sampleType: VSSampleType,
        bitsPerSample: c_int,
        subSamplingW: c_int,
        subSamplingH: c_int,
        core: *mut VSCore,
    ) -> u32,
    /// non-zero return value on success
    pub getVideoFormatByID:
        unsafe extern "system" fn(format: *mut VSVideoFormat, id: u32, core: *mut VSCore) -> c_int,

    // Frame request and filter getFrame functions
    /// only for external applications using the core as a library or for requesting frames
    /// in a filter constructor, do not use inside a filter's getFrame function
    pub getFrame: unsafe extern "system" fn(
        n: c_int,
        node: *mut VSNode,
        errorMsg: *mut c_char,
        bufSize: c_int,
    ) -> *const VSFrame,
    /// only for external applications using the core as a library or for requesting frames
    /// in a filter constructor, do not use inside a filter's getFrame function
    pub getFrameAsync: unsafe extern "system" fn(
        n: c_int,
        node: *mut VSNode,
        callback: VSFrameDoneCallback,
        userData: *mut c_void,
    ),
    /// only use inside a filter's getFrame function
    pub getFrameFilter: unsafe extern "system" fn(
        n: c_int,
        node: *mut VSNode,
        frameCtx: *mut VSFrameContext,
    ) -> *const VSFrame,
    /// only use inside a filter's getFrame function
    pub requestFrameFilter:
        unsafe extern "system" fn(n: c_int, node: *mut VSNode, frameCtx: *mut VSFrameContext),
    /// only use inside a filter's getFrame function, unless this function is called
    /// a requested frame is kept in memory until the end of processing the current frame
    pub releaseFrameEarly:
        unsafe extern "system" fn(node: *mut VSNode, n: c_int, frameCtx: *mut VSFrameContext),
    /// used to store intermediate frames in cache, useful for filters where random access is slow,
    /// must call [`setLinearFilter`](Self::setLinearFilter) on the node
    /// before using or the result is undefined
    pub cacheFrame:
        unsafe extern "system" fn(frame: *const VSFrame, n: c_int, frameCtx: *mut VSFrameContext),
    /// used to signal errors in the filter getFrame function
    pub setFilterError:
        unsafe extern "system" fn(errorMessage: *const c_char, frameCtx: *mut VSFrameContext),

    // External functions
    pub createFunction: unsafe extern "system" fn(
        func: VSPublicFunction,
        userData: *mut c_void,
        free: VSFreeFunctionData,
        core: *mut VSCore,
    ) -> *mut VSFunction,
    pub freeFunction: unsafe extern "system" fn(f: *mut VSFunction),
    pub addFunctionRef: unsafe extern "system" fn(f: *mut VSFunction) -> *mut VSFunction,
    pub callFunction:
        unsafe extern "system" fn(func: *mut VSFunction, in_: *const VSMap, out: *mut VSMap),

    // Map and property access functions
    pub createMap: unsafe extern "system" fn() -> *mut VSMap,
    pub freeMap: unsafe extern "system" fn(map: *mut VSMap),
    pub clearMap: unsafe extern "system" fn(map: *mut VSMap),
    /// copies all values in src to dst, if a key already exists in dst it's replaced
    pub copyMap: unsafe extern "system" fn(src: *const VSMap, dst: *mut VSMap),

    /// used to signal errors outside filter getFrame function
    pub mapSetError: unsafe extern "system" fn(map: *mut VSMap, errorMessage: *const c_char),
    /// used to query errors, returns 0 if no error
    pub mapGetError: unsafe extern "system" fn(map: *const VSMap) -> *const c_char,

    pub mapNumKeys: unsafe extern "system" fn(map: *const VSMap) -> c_int,
    pub mapGetKey: unsafe extern "system" fn(map: *const VSMap, index: c_int) -> *const c_char,
    pub mapDeleteKey: unsafe extern "system" fn(map: *mut VSMap, key: *const c_char) -> c_int,
    /// returns -1 if a key doesn't exist
    pub mapNumElements: unsafe extern "system" fn(map: *const VSMap, key: *const c_char) -> c_int,
    pub mapGetType:
        unsafe extern "system" fn(map: *const VSMap, key: *const c_char) -> VSPropertyType,
    pub mapSetEmpty:
        unsafe extern "system" fn(map: *mut VSMap, key: *const c_char, type_: c_int) -> c_int,

    pub mapGetInt: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> i64,
    pub mapGetIntSaturated: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> c_int,
    pub mapGetIntArray: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        error: *mut c_int,
    ) -> *const i64,
    pub mapSetInt: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        i: i64,
        append: c_int,
    ) -> c_int,
    pub mapSetIntArray: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        i: *const i64,
        size: c_int,
    ) -> c_int,

    pub mapGetFloat: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> c_double,
    pub mapGetFloatSaturated: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> c_float,
    pub mapGetFloatArray: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        error: *mut c_int,
    ) -> *const c_double,
    pub mapSetFloat: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        d: c_double,
        append: c_int,
    ) -> c_int,
    pub mapSetFloatArray: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        d: *const c_double,
        size: c_int,
    ) -> c_int,

    pub mapGetData: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> *const c_char,
    pub mapGetDataSize: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> c_int,
    pub mapGetDataTypeHint: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> VSDataTypeHint,
    pub mapSetData: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        data: *const c_char,
        size: c_int,
        type_: c_int,
        append: c_int,
    ) -> c_int,

    pub mapGetNode: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> *mut VSNode,
    /// returns 0 on success
    pub mapSetNode: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        node: *mut VSNode,
        append: c_int,
    ) -> c_int,
    /// always consumes the reference, even on error
    pub mapConsumeNode: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        node: *mut VSNode,
        append: c_int,
    ) -> c_int,

    pub mapGetFrame: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> *const VSFrame,
    pub mapSetFrame: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        f: *const VSFrame,
        append: c_int,
    ) -> c_int,
    pub mapConsumeFrame: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        f: *const VSFrame,
        append: c_int,
    ) -> c_int,

    pub mapGetFunction: unsafe extern "system" fn(
        map: *const VSMap,
        key: *const c_char,
        index: c_int,
        error: *mut c_int,
    ) -> *mut VSFunction,
    /// returns 0 on success
    pub mapSetFunction: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        func: *mut VSFunction,
        append: c_int,
    ) -> c_int,
    /// always consumes the reference, even on error
    pub mapConsumeFunction: unsafe extern "system" fn(
        map: *mut VSMap,
        key: *const c_char,
        func: *mut VSFunction,
        append: c_int,
    ) -> c_int,

    // Plugin and plugin function related
    /// non-zero return value on success
    pub registerFunction: unsafe extern "system" fn(
        name: *const c_char,
        args: *const c_char,
        returnType: *const c_char,
        argsFunc: VSPublicFunction,
        functionData: *mut c_void,
        plugin: *mut VSPlugin,
    ) -> c_int,
    pub getPluginByID:
        unsafe extern "system" fn(identifier: *const c_char, core: *mut VSCore) -> *mut VSPlugin,
    pub getPluginByNamespace:
        unsafe extern "system" fn(ns: *const c_char, core: *mut VSCore) -> *mut VSPlugin,
    /// pass NULL to get the first plugin
    pub getNextPlugin:
        unsafe extern "system" fn(plugin: *mut VSPlugin, core: *mut VSCore) -> *mut VSPlugin,
    /// pass NULL to get the first plugin function
    pub getPluginName: unsafe extern "system" fn(plugin: *mut VSPlugin) -> *const c_char,
    pub getPluginID: unsafe extern "system" fn(plugin: *mut VSPlugin) -> *const c_char,
    pub getPluginNamespace: unsafe extern "system" fn(plugin: *mut VSPlugin) -> *const c_char,
    pub getNextPluginFunction: unsafe extern "system" fn(
        func: *mut VSPluginFunction,
        plugin: *mut VSPlugin,
    ) -> *mut VSPluginFunction,
    pub getPluginFunctionByName: unsafe extern "system" fn(
        name: *const c_char,
        plugin: *mut VSPlugin,
    ) -> *mut VSPluginFunction,
    pub getPluginFunctionName:
        unsafe extern "system" fn(func: *mut VSPluginFunction) -> *const c_char,
    /// returns an argument format string
    pub getPluginFunctionArguments:
        unsafe extern "system" fn(func: *mut VSPluginFunction) -> *const c_char,
    /// returns an argument format string
    pub getPluginFunctionReturnType:
        unsafe extern "system" fn(func: *mut VSPluginFunction) -> *const c_char,
    /// the full path to the loaded library file containing the plugin entry point
    pub getPluginPath: unsafe extern "system" fn(plugin: *const VSPlugin) -> *const c_char,
    pub getPluginVersion: unsafe extern "system" fn(plugin: *const VSPlugin) -> c_int,
    /// user must free the returned [`VSMap`]
    pub invoke: unsafe extern "system" fn(
        plugin: *mut VSPlugin,
        name: *const c_char,
        args: *const VSMap,
    ) -> *mut VSMap,

    // Core and information
    pub createCore: unsafe extern "system" fn(flags: VSCoreCreationFlags) -> *mut VSCore,
    /// only call this function after all node,
    /// frame and function references belonging to the core have been freed
    pub freeCore: unsafe extern "system" fn(core: *mut VSCore),
    /// the total cache size at which vapoursynth more aggressively tries to reclaim memory,
    /// it is not a hard limit
    pub setMaxCacheSize: unsafe extern "system" fn(bytes: i64, core: *mut VSCore) -> i64,
    /// setting threads to 0 means automatic detection
    pub setThreadCount: unsafe extern "system" fn(threads: c_int, core: *mut VSCore) -> c_int,
    pub getCoreInfo: unsafe extern "system" fn(core: *mut VSCore, info: *mut VSCoreInfo),
    pub getAPIVersion: unsafe extern "system" fn() -> c_int,

    // Message handler
    pub logMessage:
        unsafe extern "system" fn(msgType: c_int, msg: *const c_char, core: *mut VSCore),
    /// `free` and `userData` can be NULL, returns a handle that can be passed to
    /// [`removeLogHandler()`](Self::removeLogHandler)
    pub addLogHandler: unsafe extern "system" fn(
        handler: VSLogHandler,
        free: Option<VSLogHandlerFree>,
        userData: *mut c_void,
        core: *mut VSCore,
    ) -> *mut VSLogHandle,
    /// returns non-zero if successfully removed
    pub removeLogHandler:
        unsafe extern "system" fn(handle: *mut VSLogHandle, core: *mut VSCore) -> c_int,

    // Graph information
    /*
     * These functions only exist to retrieve internal details for debug purposes and
     * graph visualization They will only only work properly when used on a core created
     * with ccfEnableGraphInspection and are not safe to use concurrently with frame requests
     * or other API functions. Because of this they are unsuitable for use in plugins and filters.
     */
    #[cfg(feature = "vs-graph")]
    /// level=0 returns the name of the function that created the filter,
    /// specifying a higher level will retrieve the function above that
    /// invoked it or `NULL` if a non-existent level is requested
    pub getNodeCreationFunctionName:
        unsafe extern "system" fn(node: *mut VSNode, level: c_int) -> *const c_char,
    #[cfg(feature = "vs-graph")]
    /// level=0 returns a copy of the arguments passed to the function that created the filter,
    /// returns `NULL` if a non-existent level is requested
    pub getNodeCreationFunctionArguments:
        unsafe extern "system" fn(node: *mut VSNode, level: c_int) -> *const VSMap,
    #[cfg(feature = "vs-graph")]
    /// the name passed to `create*Filter*`
    pub getNodeName: unsafe extern "system" fn(node: *mut VSNode) -> *const c_char,
    #[cfg(feature = "vs-graph")]
    pub getNodeFilterMode: unsafe extern "system" fn(node: *mut VSNode) -> VSFilterMode,
    #[cfg(feature = "vs-graph")]
    /// time spent processing frames in nanoseconds
    pub getNodeFilterTime: unsafe extern "system" fn(node: *mut VSNode) -> i64,
    #[cfg(feature = "vs-graph")]
    pub getNodeDependencies:
        unsafe extern "system" fn(node: *mut VSNode) -> *const VSFilterDependency,
    #[cfg(feature = "vs-graph")]
    pub getNumNodeDependencies: unsafe extern "system" fn(node: *mut VSNode) -> c_int,
}

extern "system" {
    pub fn getVapourSynthAPI(version: c_int) -> *const VSAPI;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout() {
        assert_eq!(
            std::mem::size_of::<VSPresetFormat>(),
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
    }
}

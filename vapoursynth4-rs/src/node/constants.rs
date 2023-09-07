use crate::ffi;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum FilterMode {
    /// Completely parallel execution. Multiple threads will call a filter's "getFrame" function,
    /// to fetch several frames in parallel.
    Parallel,
    /// For filters that are serial in nature but can request in advance one or more frames
    /// they need. A filter's "getFrame" function will be called from multiple threads at a time
    /// with activation reason [`ActivationReason::Initial`],
    /// but only one thread will call it with activation reason
    /// [`ActivationReason::AllFramesReady`] at a time.
    ParallelRequests,
    /// Only one thread can call the filter's "getFrame" function at a time.
    /// Useful for filters that modify or examine their internal state to
    /// determine which frames to request.
    ///
    /// While the "getFrame" function will only run in one thread at a time,
    /// the calls can happen in any order. For example, it can be called with reason
    /// [`ActivationReason::Initial`] for frame 0, then again with reason
    /// [`ActivationReason::Initial`] for frame 1,
    /// then with reason [`ActivationReason::AllFramesReady`]  for frame 0.
    Unordered,
    /// For compatibility with other filtering architectures.
    /// *DO NOT USE IN NEW FILTERS*. The filter's "getFrame" function only ever gets called from
    /// one thread at a time. Unlike [`FilterMode::Unordered`],
    /// only one frame is processed at a time.
    FrameState,
}

impl From<FilterMode> for ffi::VSFilterMode {
    fn from(mode: FilterMode) -> Self {
        use ffi::VSFilterMode as vm;
        use FilterMode as m;

        match mode {
            m::Parallel => vm::fmParallel,
            m::ParallelRequests => vm::fmParallelRequests,
            m::Unordered => vm::fmUnordered,
            m::FrameState => vm::fmFrameState,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum CacheMode {
    /// Cache is enabled or disabled based on the reported request patterns
    /// and number of consumers.
    Auto = -1,
    /// Never cache anything.
    ForceDisable = 0,
    /// Never cache anything.
    ForceEnable = 1,
}

impl From<CacheMode> for ffi::VSCacheMode {
    fn from(mode: CacheMode) -> Self {
        use ffi::VSCacheMode as vm;
        use CacheMode as m;

        match mode {
            m::Auto => vm::cmAuto,
            m::ForceDisable => vm::cmForceDisable,
            m::ForceEnable => vm::cmForceEnable,
        }
    }
}

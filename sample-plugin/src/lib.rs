mod dither;
mod splice;

use vapoursynth4_rs::declare_plugin;

use dither::DitherFilter;
use splice::SpliceFilter;

declare_plugin!(
    c"com.example.sample",
    c"sample",
    c"VapourSynth Filter Sample",
    (1, 0),
    vapoursynth4_rs::VAPOURSYNTH_API_VERSION,
    0,
    (SpliceFilter, None),
    (DitherFilter, None)
);


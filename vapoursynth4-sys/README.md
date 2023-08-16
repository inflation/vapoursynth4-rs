# `vapoursynth4-sys`

[![Crates](https://img.shields.io/crates/v/vapoursynth4-sys.svg)][1]
[![Documentation](https://docs.rs/vapoursynth4-sys/badge.svg)][2]

[1]: https://crates.io/crates/vapoursynth4-sys
[2]: https://docs.rs/vapoursynth4-sys

Raw bindings to [VapourSynth][3]. Special thanks to [Ivan Molodetskikh][4] for 
their work on the original bindings.

[3]: https://github.com/vapoursynth/vapoursynth
[4]: https://github.com/YaLTeR/vapoursynth-rs

Check out [vapoursynth4-rs](https://crates.io/crates/vapoursynth4-rs) for a safe Rust wrapper.

## Supported Versions

All VapourSynth and VSScript API versions starting with 4.0 are supported.
By default, the crates use the latest API version available.  To use a specific version, 
disable the default feature and enable the corresponding Cargo feature:

- `vapoursynth-api-40` for VapourSynth API 4.0 (R55)
- `vsscript-api-40` for VSScript API 4.0
- `vsscript-api-41` for VSScript API 4.1

## Building

Make sure you have the corresponding libraries available if you enable the linking features.
You can use the `VAPOURSYNTH_LIB_DIR` environment variable to specify
a custom directory with the library files.

On Windows the easiest way is to use the VapourSynth installer (make sure the VapourSynth SDK
is checked). Set `VAPOURSYNTH_LIB_DIR` to `<path to the VapourSynth installation>\sdk\lib64`
or `<...>\lib32`, depending on the target bit count.

## License

Licensed under [MPL-2.0](LICENSE) or at http://mozilla.org/MPL/2.0/.

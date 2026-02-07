# `vapoursynth4-rs`

[![Crates](https://img.shields.io/crates/v/vapoursynth4-rs.svg)][1]
[![Documentation](https://docs.rs/vapoursynth4-rs/badge.svg)][2]
[![dependency status](https://deps.rs/repo/github/inflation/vapoursynth4-rs/status.svg)][3]
[![CI](https://github.com/inflation/vapoursynth4-rs/workflows/CI/badge.svg)][4]
[![License: MPL-2.0](https://img.shields.io/crates/l/vapoursynth4-rs)][5]

[1]: https://crates.io/crates/vapoursynth4-rs
[2]: https://docs.rs/vapoursynth4-rs
[3]: https://deps.rs/repo/github/inflation/vapoursynth4-rs
[4]: https://github.com/inflation/vapoursynth4-rs/actions?query=workflow%3ACI
[5]: https://github.com/inflation/vapoursynth4-rs/blob/master/LICENSE

Safe wrapper to [VapourSynth][6]. Special thanks to [Ivan Molodetskikh][7] for
their work on the original bindings.

[6]: https://github.com/vapoursynth/vapoursynth
[7]: https://github.com/YaLTeR/vapoursynth-rs

Check out [`vapoursynth4-sys`](https://crates.io/crates/vapoursynth4-sys) for the raw binding.
Check out [`vapoursynth4-sys`](https://crates.io/crates/vapoursynth4-sys) for the raw binding.

## Supported Versions

All VapourSynth and VSScript API versions starting with 4.0 are supported.
By default, the crates use the latest API version available.
To use a specific version,
disable the default feature and enable the corresponding Cargo feature:

- `vs-41` for VapourSynth API 4.1 (R66)
- `vsscript` for VSScript API 4.0
- `vsscript-42` for VSScript API 4.1

## Building

Make sure you have the corresponding libraries available if you enable the
linking features. You can use the `VAPOURSYNTH_LIB_DIR` environment variable to
specify a custom directory with the library files.

On Windows the easiest way is to use the VapourSynth installer (make sure the
VapourSynth SDK is checked). Set `VAPOURSYNTH_LIB_DIR` to
`<path to the VapourSynth installation>\sdk\lib64` or `<...>\lib32`, depending
on the target.

## License

Licensed under [MPL-2.0](LICENSE) or at <http://mozilla.org/MPL/2.0/>.

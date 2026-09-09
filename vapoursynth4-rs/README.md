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

## Supported Versions

All VapourSynth and VSScript API versions starting with 4.0 are supported,
up to VapourSynth API 4.2 and VSScript API 4.3 (R79).
By default, the crate uses the latest API version available.
To use a specific version,
disable the default features and enable the corresponding Cargo feature:

- `vs-41` for VapourSynth API 4.1 (R66)
- `vs-42` for VapourSynth API 4.2 (R74)
- `vsscript` for VSScript API 4.1, the base version
- `vsscript-42` for VSScript API 4.2 (R69)
- `vsscript-43` for VSScript API 4.3 (R79)
- `vs-graph` for the experimental graph inspection API (implies `vs-42`)

Each version feature enables the ones below it. Note that API 4.2 replaces the
`_ColorRange` frame property with `_Range`, whose two values are the other way
around; enabling `vs-42` deprecates `VSColorRange` in favour of `VSRange`.

## Building

Linking is optional and enabled with the `link-vs` and `link-vsscript` features.
If you enable them, make sure the corresponding libraries are available.

The simplest way to get them on any platform is the official wheel, which is the
same payload the Windows installer and the portable zip ship since R74:

```sh
pip install vapoursynth==79
```

Use the `VAPOURSYNTH_LIB_PATH` environment variable to point at the directory
holding the library files.

On Windows no import library is needed, and none is shipped any more: the DLLs
are bound directly by name (`libvapoursynth.dll` and `vsscript.dll`), so it is
enough for them to be findable on `PATH` at run time.

On Linux and macOS the wheel ships only the runtime sonames
(`libvapoursynth.so.4`, `libvsscript.so`), so linking against it needs the usual
development symlinks — see `scripts/install-vs.sh` for the exact set.

## License

Licensed under [MPL-2.0](LICENSE) or at <http://mozilla.org/MPL/2.0/>.

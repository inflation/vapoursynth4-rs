# `vapoursynth4-sys`

[![Crates](https://img.shields.io/crates/v/vapoursynth4-sys.svg)][1]
[![Documentation](https://docs.rs/vapoursynth4-sys/badge.svg)][2]

[1]: https://crates.io/crates/vapoursynth4-sys
[2]: https://docs.rs/vapoursynth4-sys

Raw bindings to [VapourSynth][3]. Special thanks to [Ivan Molodetskikh][4] for
their work on the original bindings.

[3]: https://github.com/vapoursynth/vapoursynth
[4]: https://github.com/YaLTeR/vapoursynth-rs

Check out [`vapoursynth4-rs`](https://crates.io/crates/vapoursynth4-rs) for a safe Rust wrapper.

## Supported Versions

All VapourSynth and VSScript API versions starting with 4.0 are supported,
up to VapourSynth API 4.2 and VSScript API 4.3 (R79).
By default this crate enables no features, i.e. VapourSynth API 4.0.
To use a specific version, enable the corresponding Cargo feature:

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

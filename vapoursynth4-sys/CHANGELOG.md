# Changelog

## [Unreleased]

## [0.4.1+R79]

Tracks `VapourSynth` R79, adding VapourSynth API 4.2 and VSScript API 4.3.

### ⚠️ Breaking Changes

- `VSAPI::getNodeDependencies` is now `getNodeDependency` and takes the `index`
  argument the C header has always declared. The previous binding was an ABI
  mismatch.
- The API 4.1 node timing members (`getCoreNodeTiming`, `setCoreNodeTiming`,
  `getNodeProcessingTime`, `getFreedNodeProcessingTime`) are now gated on
  `vs-41`, matching the header. Without the feature they misaligned the tail of
  the struct against the real library.
- `VSSCRIPTAPI::freeScript` now returns `()` instead of `c_int`, and
  `setVariable` is renamed `setVariables` to match the header.
- On Windows the libraries are bound by DLL name with `raw-dylib`
  (`libvapoursynth.dll` and `vsscript.dll`). No `VapourSynth` release since R74
  ships an import library, so `VAPOURSYNTH_LIB_PATH` is no longer needed there.
- Version features now enable their predecessors: `vs-42` implies `vs-41`,
  `vs-graph` implies `vs-42`, and `vsscript-43` implies `vsscript-42`.
  `vs-graph` in particular must imply `vs-42`, because R79 places
  `getCoreInfo2` between the 4.1 block and the graph functions.
- API 4.2 replaces the `_ColorRange` frame property with `_Range`, which follows
  H.273 numbering — **its two values are swapped**. `VSColorRange` is deprecated
  when `vs-42` is enabled; use `VSRange`.

### ⛰️ Features

- [**breaking**] Add VapourSynth API 4.2 and VSScript API 4.3 - ([a5ae4b8](https://github.com/inflation/vapoursynth4-rs/commit/a5ae4b8307a6fbb8ebf1f9030cb4bb47d51161ad))

  Adds `getCoreInfo2` and `VSCoreInfo2`, the `getNodeCreationPluginID` and
  `getNodeCreationPluginNS` graph functions, `getVSScriptAPILastError`,
  `VSRange`, `ccfEnableFrameRefDebug`, `rpFrameReuseLastOnly`, and the YUV410,
  YUV411 and YUV440 presets for 16-bit integer, half and single precision.

### 🐛 Bug Fixes

- [**breaking**] Bind Windows libraries with raw-dylib - ([f682450](https://github.com/inflation/vapoursynth4-rs/commit/f6824502d63a5d2660f065693d5e849660a25403))
- [**breaking**] Correct VSAPI and VSSCRIPTAPI ABI mismatches - ([c4ff5a9](https://github.com/inflation/vapoursynth4-rs/commit/c4ff5a99f7a834d29232cfc6c8f11101de94bb7f))

### 📚 Documentation

- Document API 4.2, VSScript 4.3 and the wheel-based setup - ([ea2e82f](https://github.com/inflation/vapoursynth4-rs/commit/ea2e82f0ee3b5c30fa2d4f50a5a17216c11f1eae))

### 🧪 Testing

- Assert VSAPI layout matches the installed library - ([beddd00](https://github.com/inflation/vapoursynth4-rs/commit/beddd0084b58300ae1e0b519f4f7f493f4d3a7e0))

### ⚙️ Miscellaneous Tasks

- Bump `vapoursynth4-sys` to 0.4.1+R79 and `vapoursynth4-rs` to 0.5.1 - ([52106f1](https://github.com/inflation/vapoursynth4-rs/commit/52106f1ec1388bb0c2bbe9434d411b1cfa61a066))
- Drop the unused optional `cc` dependency

## [0.3.1](https://github.com/inflation/vapoursynth4-rs/compare/vapoursynth4-sys-v0.3.0...vapoursynth4-sys-v0.3.1)

### 🚜 Refactor

- Update to Rust Edition 2024 - ([ed2da0f](https://github.com/inflation/vapoursynth4-rs/commit/ed2da0fa3ed27f2c07ba8993797948f011012b1c))

### 🧪 Testing

- Test plugin with library - ([15ffab3](https://github.com/inflation/vapoursynth4-rs/commit/15ffab39e967ad298211425c27d567feec92884b))

## [0.3.0](https://github.com/inflation/vapoursynth4-rs/compare/vapoursynth4-sys-v0.2.0...vapoursynth4-sys-v0.3.0)

### ⛰️ Features

- *(rs)* :recycle: Change VSAPI usage - ([6f94db0](https://github.com/inflation/vapoursynth4-rs/commit/6f94db0397cffe55937e288d795c3f9bdefcd209))
- *(sys)* :building_construction: Make linking optional for plugins - ([cd0c189](https://github.com/inflation/vapoursynth4-rs/commit/cd0c1892d8fdd23f349126b524bced5b6fcd8bfa))
- Constant validation of KeyStr - ([e9d6e9b](https://github.com/inflation/vapoursynth4-rs/commit/e9d6e9b40dd5860175f7773fb4c65571d6788b18))
- :construction_worker: Initial CI config - ([266ebf5](https://github.com/inflation/vapoursynth4-rs/commit/266ebf50c7a1c9d6e61700f98fe9f2cb5c261100))

### 🚜 Refactor

- Carry VSAPI in structs - ([659d840](https://github.com/inflation/vapoursynth4-rs/commit/659d840e303ea2e46f04b2888b687d81f17a2dac))
- Use `system-unwind` for FFI functions - ([68de7bc](https://github.com/inflation/vapoursynth4-rs/commit/68de7bcf6637573cf87f478ee081126153cbbc78))
- ♻️ Change API to use its interface in Core - ([00d253b](https://github.com/inflation/vapoursynth4-rs/commit/00d253b993f7ca0d38783c675c49386852808ba3))

### 📦 Dependencies

- Update `VapourSynth` to R70 - ([12e0e00](https://github.com/inflation/vapoursynth4-rs/commit/12e0e00e8b252a3e6525373f175a820ab90a0724))

### ⚙️ Miscellaneous Tasks

- Remove unused proc macro crate - ([4aaefef](https://github.com/inflation/vapoursynth4-rs/commit/4aaefeffc6f6040924b4d0c91dc0069a7272c90c))
- Add release-plz - ([fbd18e5](https://github.com/inflation/vapoursynth4-rs/commit/fbd18e559c79379e9b922d96fafb277a5f61da1f))
- 👷 Add windows testing - ([02a3c44](https://github.com/inflation/vapoursynth4-rs/commit/02a3c44fb063791c10dbf3b3798ba017a6100e18))
- :construction_worker: Add macOS testing - ([455fe7b](https://github.com/inflation/vapoursynth4-rs/commit/455fe7b0db84a8a38920528e2e40a6c459801cdf))


# Changelog

## [Unreleased]

## [0.6.0](https://github.com/inflation/vapoursynth4-rs/compare/vapoursynth4-rs-v0.5.1...vapoursynth4-rs-v0.6.0)

### ⛰️ Features

- Add Vulkan and VSScript 4.4 APIs - ([e41ebc7](https://github.com/inflation/vapoursynth4-rs/commit/e41ebc7da800507d39875732daa8fa073baabd05))
- Update API to VapourSynth R80 - ([4dd754a](https://github.com/inflation/vapoursynth4-rs/commit/4dd754aecc3630cf911f5c10abc922ec35d4eddf))

### 🐛 Bug Fixes

- Transfer core ownership to VSScript - ([14f25ac](https://github.com/inflation/vapoursynth4-rs/commit/14f25acb84c83a1208ae4b161ba3f3a3943f235f))
- Check native format query results - ([6b02eb0](https://github.com/inflation/vapoursynth4-rs/commit/6b02eb0df25f1ee6f6ffd12f8af87373f8c50f6d))
- Require unsafe for raw frame assembly - ([8fc3670](https://github.com/inflation/vapoursynth4-rs/commit/8fc3670d4058f6f6a2a981bb0311cf1550cd3243))
- Borrow filter registration data across calls - ([04212e5](https://github.com/inflation/vapoursynth4-rs/commit/04212e5eabd14a33de9d3019f7540f76fe699c96))
- Make Rust filter callbacks safe to invoke - ([e9c02e2](https://github.com/inflation/vapoursynth4-rs/commit/e9c02e2ad140cce88f2395ac1dbee2e1579e89c5))
- Separate borrowed cores from ownership - ([682d1f7](https://github.com/inflation/vapoursynth4-rs/commit/682d1f736d6f5b81fd7de83b67a7a8718bbacac0))
- Enforce exclusive map and frame mutation - ([ed76e9a](https://github.com/inflation/vapoursynth4-rs/commit/ed76e9a2ee0d685375bb2d52997dc3ff55dbf9db))
- Prevent cloning owned scripts - ([1ac4fcf](https://github.com/inflation/vapoursynth4-rs/commit/1ac4fcf287e7bae4bbe9dae63ce96f1d7faf65c5))

### 🚜 Refactor

- Name the shared core handle and keep tests internal - ([bcbbb80](https://github.com/inflation/vapoursynth4-rs/commit/bcbbb8056453364fc8753cd7bcda52607207e5e1))

### 🧪 Testing

- Cover native wrappers and fix exposed FFI bugs - ([ce90536](https://github.com/inflation/vapoursynth4-rs/commit/ce905366a6a65e9a0a015eb0406aadffd7e609db))
- Cover audio outputs and native video filter creation - ([a259a26](https://github.com/inflation/vapoursynth4-rs/commit/a259a26783c44b009f4fdc59cd406eb80b4bef5f))
- Cover Vulkan residency in Codecov - ([9af615c](https://github.com/inflation/vapoursynth4-rs/commit/9af615c6c39be774a20b2e45edd9f1a421a5edf6))

### ⚙️ Miscellaneous Tasks

- Exercise Vulkan with Mesa Lavapipe - ([ead1766](https://github.com/inflation/vapoursynth4-rs/commit/ead1766ccf0b2a33f9b72948d9dde763e173fd5d))


- Bind the R80 Vulkan API behind `vulkan`, using Ash Vulkan types.
- Add VSScript API 4.4 and GPU-aware output and variable accessors.

- Require Rust 1.95 and use `cfg_select!` for conditional API selection.

- Target VapourSynth API 4.3 (R80) by default, with `vs-43` feature forwarding.
- Add frame/node residency queries and `Filter::FLAGS` for GPU output.
- Fix video filter construction to return a node only on success.
- Update CI runtime installation to R80.

## [0.5.1]

Tracks `VapourSynth` R79 via `vapoursynth4-sys` 0.4.1+R79.

### ⚠️ Breaking Changes

- Default features are now `vs-42`, `vsscript` and `vsscript-43`, so the crate
  targets VapourSynth API 4.2 out of the box and requires R74 or newer at run
  time. Pin the older features explicitly to keep the previous behaviour.
- On Windows the libraries are bound by DLL name with `raw-dylib`, so no import
  library or SDK install is needed, and `VAPOURSYNTH_LIB_PATH` is not consulted
  there. See the `vapoursynth4-sys` 0.4.1+R79 notes for the full list.

### ⛰️ Features

- Wrap the new API 4.2 and VSScript 4.3 functions - ([4f81484](https://github.com/inflation/vapoursynth4-rs/commit/4f8148486d2a3c5afc532539d0b6232ac43f1a96))

  Adds `Core::get_info2()`, `CoreBuilder::enable_frame_ref_debug()`, graph
  inspection on `Node` (`creation_function_name()`, `creation_plugin_id()`,
  `creation_plugin_namespace()`, `creation_function_arguments()`),
  `VssApi::last_error()`, and a `Range` alias for the new `_Range` numbering.

- [**breaking**] Add VapourSynth API 4.2 and VSScript API 4.3 - ([a5ae4b8](https://github.com/inflation/vapoursynth4-rs/commit/a5ae4b8307a6fbb8ebf1f9030cb4bb47d51161ad))

### 📚 Documentation

- Document API 4.2, VSScript 4.3 and the wheel-based setup - ([ea2e82f](https://github.com/inflation/vapoursynth4-rs/commit/ea2e82f0ee3b5c30fa2d4f50a5a17216c11f1eae))

### 📦 Dependencies

- Update bon and thiserror requirements - ([eba6f8e](https://github.com/inflation/vapoursynth4-rs/commit/eba6f8e1d22ad350532fbf1c693d3d9290d5f7db))

### ⚙️ Miscellaneous Tasks

- Bump `vapoursynth4-sys` to 0.4.1+R79 and `vapoursynth4-rs` to 0.5.1 - ([52106f1](https://github.com/inflation/vapoursynth4-rs/commit/52106f1ec1388bb0c2bbe9434d411b1cfa61a066))

## [0.4.0](https://github.com/inflation/vapoursynth4-rs/compare/vapoursynth4-rs-v0.3.0...vapoursynth4-rs-v0.4.0)

### 🚜 Refactor

- Update to Rust Edition 2024 - ([ed2da0f](https://github.com/inflation/vapoursynth4-rs/commit/ed2da0fa3ed27f2c07ba8993797948f011012b1c))

### 🧪 Testing

- Test plugin with library - ([15ffab3](https://github.com/inflation/vapoursynth4-rs/commit/15ffab39e967ad298211425c27d567feec92884b))

### 📦 Dependencies

- (deps): update const-str requirement from 0.5.6 to 0.6.1 - ([8581831](https://github.com/inflation/vapoursynth4-rs/commit/85818318307a2a5f2540ac3a71d8bf85bb1c6f02))

## [0.3.0](https://github.com/inflation/vapoursynth4-rs/compare/vapoursynth4-rs-v0.2.0...vapoursynth4-rs-v0.3.0)

### ⛰️ Features

- *(rs)* :recycle: Change VSAPI usage - ([6f94db0](https://github.com/inflation/vapoursynth4-rs/commit/6f94db0397cffe55937e288d795c3f9bdefcd209))
- *(sys)* :building_construction: Make linking optional for plugins - ([cd0c189](https://github.com/inflation/vapoursynth4-rs/commit/cd0c1892d8fdd23f349126b524bced5b6fcd8bfa))
- Constant validation of KeyStr - ([e9d6e9b](https://github.com/inflation/vapoursynth4-rs/commit/e9d6e9b40dd5860175f7773fb4c65571d6788b18))

### 🚜 Refactor

- *(rs)* :recycle: Move node::filter::internal to a separate file - ([1fca75b](https://github.com/inflation/vapoursynth4-rs/commit/1fca75b69c3434cb2723bb6b60ce642f7100805a))
- Carry VSAPI in structs - ([659d840](https://github.com/inflation/vapoursynth4-rs/commit/659d840e303ea2e46f04b2888b687d81f17a2dac))
- Use `bon` for typed builder - ([4228075](https://github.com/inflation/vapoursynth4-rs/commit/4228075c7f31d2dc48dafd8159a7c7646faca2f0))
- Use `system-unwind` for FFI functions - ([68de7bc](https://github.com/inflation/vapoursynth4-rs/commit/68de7bcf6637573cf87f478ee081126153cbbc78))
- Simplify maps to just `Map` and `MapRef` - ([74c26f9](https://github.com/inflation/vapoursynth4-rs/commit/74c26f9597706bf9c36119fed9c7389ea34be298))
- ♻️ Change API to use its interface in Core - ([00d253b](https://github.com/inflation/vapoursynth4-rs/commit/00d253b993f7ca0d38783c675c49386852808ba3))

### ⚙️ Miscellaneous Tasks

- Remove unused proc macro crate - ([4aaefef](https://github.com/inflation/vapoursynth4-rs/commit/4aaefeffc6f6040924b4d0c91dc0069a7272c90c))
- Add release-plz - ([fbd18e5](https://github.com/inflation/vapoursynth4-rs/commit/fbd18e559c79379e9b922d96fafb277a5f61da1f))
- :arrow_up: (deps): update thiserror requirement - ([6cd2bea](https://github.com/inflation/vapoursynth4-rs/commit/6cd2bead8e5a2b5116a7c801319523d086bb262f))
- :construction_worker: Add macOS testing - ([455fe7b](https://github.com/inflation/vapoursynth4-rs/commit/455fe7b0db84a8a38920528e2e40a6c459801cdf))


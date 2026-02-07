/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

// VSScript4.h
//! `VSScript` provides a convenient wrapper for VapourSynth’s scripting interface(s),
//! allowing the evaluation of `VapourSynth` scripts and retrieval of output clips.
//!
//! For reasons unknown, the `VSScript` library is called `VSScript` in Windows and
//! `vapoursynth-script` everywhere else.
//!
//! At this time, `VapourSynth` scripts can be written only in Python (version 3).
//!
//! Here are a few users of the `VSScript` library:
//!
//! * [vspipe](https://github.com/vapoursynth/vapoursynth/blob/master/src/vspipe/vspipe.cpp)
//! * [vsvfw](https://github.com/vapoursynth/vapoursynth/blob/master/src/vfw/vsvfw.cpp)
//! * [an example program][1]
//! * the video player [mpv]
//!
//! [1]: https://github.com/vapoursynth/vapoursynth/blob/master/sdk/vsscript_example.c
//! [mpv]: https://github.com/mpv-player/mpv/blob/master/video/filter/vf_vapoursynth.c
//!
//! # Note
//!
//! If `libvapoursynth-script` is loaded with `dlopen()`, the `RTLD_GLOBAL` flag must be used.
//! If not, Python won’t be able to import binary modules. This is due to Python’s design.

#![cfg(feature = "vsscript")]

use std::ffi::{c_char, c_int, c_void};

use super::{VSAPI, VSCore, VSMap, VSNode, opaque_struct, vs_make_version};

pub const VSSCRIPT_API_MAJOR: u16 = 4;
pub const VSSCRIPT_API_MINOR: u16 = if cfg!(feature = "vsscript-42") { 2 } else { 1 };
pub const VSSCRIPT_API_VERSION: i32 = vs_make_version(VSSCRIPT_API_MAJOR, VSSCRIPT_API_MINOR);

opaque_struct!(
    /// A script environment. All evaluation and communication with evaluated scripts happens
    /// through a [`VSScript`] object.
    VSScript
);

/// This struct is the way to access VSScript’s public API.
#[allow(non_snake_case)]
#[repr(C)]
pub struct VSSCRIPTAPI {
    /// Returns the api version provided by vsscript.
    pub getApiVersion: unsafe extern "system-unwind" fn() -> c_int,

    /// Retrieves the [`VSAPI`] struct. Exists mostly as a convenience so
    /// the vapoursynth module doesn’t have to be explicitly loaded.
    ///
    /// This could return `NULL` if the `VapourSynth` library doesn’t
    /// provide the requested version.
    pub getVSAPI: unsafe extern "system-unwind" fn(version: c_int) -> *const VSAPI,

    /// Creates an empty script environment that can be used to evaluate scripts.
    /// Passing a pre-created core can be useful to have custom core creation flags,
    /// log callbacks or plugins pre-loaded. Passing `NULL` will automatically create
    /// a new core with default settings.
    ///
    /// Takes over ownership of the core regardless of success or failure.
    /// Returns `NULL` on error.
    pub createScript: unsafe extern "system-unwind" fn(core: *mut VSCore) -> *mut VSScript,

    /// Retrieves the `VapourSynth` core that was created in the script environment.
    /// If a `VapourSynth` core has not been created yet, it will be created now,
    /// with the default options (see the [Python Reference][1]).
    ///
    /// [1]: http://www.vapoursynth.com/doc/pythonreference.html
    ///
    /// [`VSScript`] retains ownership of the returned core object.
    ///
    /// Returns `NULL` on error.
    ///
    /// Note: The core is valid as long as the environment exists
    pub getCore: unsafe extern "system-unwind" fn(handle: *mut VSScript) -> *mut VSCore,

    /// Evaluates a script contained in a C string. Can be called multiple times on
    /// the same script environment to successively add more processing.
    ///
    /// # Arguments
    ///
    /// * `handle` - Pointer to a script environment.
    ///
    /// * `buffer` - The entire script to evaluate, as a C string.
    ///
    /// * `scriptFilename` - A name for the script, which will be displayed in error messages.
    ///   If this is `NULL`, the name "\<string\>" will be used.
    ///
    /// The special `__file__` variable will be set to `scriptFilename`'s absolute path
    /// if this is not `NULL`.
    ///
    /// Returns non-zero in case of errors. The error message can be retrieved with
    /// [`getError()`](Self::getError). If the script calls `sys.exit(code)`
    /// the exit code can be retrieved with [`getExitCode()`](Self::getExitCode).
    /// The working directory behavior can be changed by calling
    /// [`evalSetWorkingDir()`](Self::evalSetWorkingDir) before this function.
    ///
    /// Note: calling any function other than [`getError()`](Self::getError) and
    /// [`freeScript()`](Self::freeScript) on a [`VSScript`] object in the error state
    /// will result in undefined behavior.
    pub evaluateBuffer: unsafe extern "system-unwind" fn(
        handle: *mut VSScript,
        buffer: *const c_char,
        scriptFilename: *const c_char,
    ) -> c_int,

    /// Evaluates a script contained in a file. This is a convenience function
    /// which reads the script from a file for you. It will only read the first 16 MiB
    /// which should be enough for everyone.
    ///
    /// Behaves the same as [`evaluateBuffer()`](Self::evaluateBuffer).
    pub evaluateFile: unsafe extern "system-unwind" fn(
        handle: *mut VSScript,
        scriptFilename: *const c_char,
    ) -> c_int,

    /// Returns the error message from a script environment, or `NULL`, if there is no error.
    ///
    /// It is okay to pass `NULL`.
    ///
    /// `VSScript` retains ownership of the pointer and it is only guaranteed
    /// to be valid until the next vsscript operation on the handle.
    pub getError: unsafe extern "system-unwind" fn(handle: *mut VSScript) -> *const c_char,

    /// Returns the exit code if the script calls `sys.exit(code)`, or 0,
    /// if the script fails for other reasons or calls `sys.exit(0)`.
    ///
    /// It is okay to pass `NULL`.
    pub getExitCode: unsafe extern "system-unwind" fn(handle: *mut VSScript) -> c_int,

    /// Retrieves a variable from the script environment.
    ///
    /// If a `VapourSynth` core has not been created yet in the script environment,
    /// one will be created now, with the default options (see the [Python Reference][1]).
    ///
    /// [1]: http://www.vapoursynth.com/doc/pythonreference.html
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the variable to retrieve.
    ///
    /// * `dst` - Map where the variable’s value will be placed, with the key name.
    ///
    /// Returns non-zero on error.
    pub getVariable: unsafe extern "system-unwind" fn(
        handle: *mut VSScript,
        name: *const c_char,
        dst: *mut VSMap,
    ) -> c_int,

    /// Sets variables in the script environment.
    ///
    /// The variables are now available to the script.
    ///
    /// If a `VapourSynth` core has not been created yet in the script environment,
    /// one will be created now, with the default options (see the [Python Reference][1]).
    ///
    /// [1]: http://www.vapoursynth.com/doc/pythonreference.html
    ///
    /// # Arguments
    ///
    /// * `vars` - Map containing the variables to set.
    ///
    /// Returns non-zero on error.
    pub setVariable:
        unsafe extern "system-unwind" fn(handle: *mut VSScript, vars: *const VSMap) -> c_int,

    /// Retrieves a node from the script environment. A node in the script must have been
    /// marked for output with the requested `index`.
    ///
    /// The returned node has its reference count incremented by one.
    ///
    /// Returns `NULL` if there is no node at the requested index.
    pub getOutputNode:
        unsafe extern "system-unwind" fn(handle: *mut VSScript, index: c_int) -> *mut VSNode,
    /// Retrieves an alpha node from the script environment. A node with associated alpha
    /// in the script must have been marked for output with the requested `index`.
    ///
    /// The returned node has its reference count incremented by one.
    ///
    /// Returns `NULL` if there is no alpha node at the requested index.
    pub getOutputAlphaNode:
        unsafe extern "system-unwind" fn(handle: *mut VSScript, index: c_int) -> *mut VSNode,
    /// Retrieves the alternative output mode settings from the script.
    /// This value has no fixed meaning but in vspipe and vsvfw it indicates
    /// that alternate output formats should be used when multiple ones are available.
    /// It is up to the client application to define the exact meaning
    /// or simply disregard it completely.
    ///
    /// Returns 0 if there is no alt output mode set.
    pub getAltOutputMode:
        unsafe extern "system-unwind" fn(handle: *mut VSScript, index: c_int) -> c_int,

    /// Frees a script environment. `handle` is no longer usable.
    ///
    /// * Cancels any clips set for output in the script environment.
    /// * Clears any variables set in the script environment.
    /// * Clears the error message from the script environment, if there is one.
    /// * Frees the `VapourSynth` core used in the script environment, if there is one.
    /// * Since this function frees the `VapourSynth` core, it must be called only after
    ///   all frame requests are finished and all objects obtained from the script
    ///   have been freed (frames, nodes, etc).
    ///
    /// It is safe to pass `NULL`.
    pub freeScript: unsafe extern "system-unwind" fn(handle: *mut VSScript) -> c_int,

    /// Set whether or not the working directory is temporarily changed to the same location
    /// as the script file when [`evaluateFile()`](Self::evaluateFile) is called. Off by default.
    pub evalSetWorkingDir:
        unsafe extern "system-unwind" fn(handle: *mut VSScript, setCWD: c_int) -> c_void,

    /// Write a list of set output index values to dst but at most size values.
    /// Always returns the total number of available output index values.
    #[cfg(feature = "vsscript-42")]
    pub getAvailableOutputNodes: unsafe extern "system-unwind" fn(
        handle: *mut VSScript,
        size: c_int,
        dst: *mut c_int,
    ) -> c_int,
}

#[cfg(feature = "link-vsscript")]
#[cfg_attr(target_os = "windows", link(name = "VSScript"))]
#[cfg_attr(not(target_os = "windows"), link(name = "vapoursynth-script"))]
unsafe extern "system-unwind" {
    /// Returns a struct containing function pointer for the api.
    /// Will return `NULL` is the specified version isn’t supported.
    ///
    /// It is recommended to always pass [`VSSCRIPT_API_VERSION`].
    pub fn getVSScriptAPI(version: c_int) -> *const VSSCRIPTAPI;
}

/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

//! VSScript4.h

#![cfg(feature = "vsscript")]

use std::ffi::*;

use super::*;

opaque_struct!(VSScript);

#[repr(C)]
pub struct VSSCRIPTAPI {
    /// Returns the highest supported [`VSSCRIPT_API_VERSION`]
    pub getApiVersion: unsafe extern "system" fn() -> c_int,

    /// Convenience function for retrieving a [`VSAPI`] pointer without having to use
    /// the VapourSynth library. Always pass [`VAPOURSYNTH_API_VERSION`]
    pub getVSAPI: unsafe extern "system" fn(version: c_int) -> *const VSAPI,

    /// Providing a pre-created core is useful for setting core creation flags,
    /// log callbacks, preload specific plugins and many other things.
    /// You must create a [`VSScript`] object before evaluating a script.
    /// Always takes ownership of the core even on failure. Returns `NULL` on failure.
    /// Pass `NULL` to have a core automatically created with the default options.
    pub createScript: unsafe extern "system" fn(core: *mut VSCore) -> *mut VSScript,

    /// The core is valid as long as the environment exists, return `NULL` on error
    pub getCore: unsafe extern "system" fn(handle: *mut VSScript) -> *mut VSCore,

    /// Evaluates a script passed in the buffer argument. The `scriptFilename` is only used for
    /// display purposes. In Python, it means that the main module
    /// won't be unnamed in error messages.
    ///
    /// Returns 0 on success.
    ///
    /// Note: calling any function other than [`getError()`](Self::getError) and
    /// [`freeScript()`](Self::freeScript) on a [`VSScript`] object in the error state
    /// will result in undefined behavior.
    pub evaluateBuffer: unsafe extern "system" fn(
        handle: *mut VSScript,
        buffer: *const c_char,
        scriptFilename: *const c_char,
    ) -> c_int,

    /// Convenience version of the above function that loads the script from `scriptFilename`
    /// and passes as the buffer to `evaluateBuffer`
    pub evaluateFile:
        unsafe extern "system" fn(handle: *mut VSScript, scriptFilename: *const c_char) -> c_int,

    /// Returns `NULL` on success, otherwise an error message
    pub getError: unsafe extern "system" fn(handle: *mut VSScript) -> *const c_char,

    /// Returns the script's reported exit code
    pub getExitCode: unsafe extern "system" fn(handle: *mut VSScript) -> c_int,

    /// Fetches a variable of any [`VSMap`] storable type set in a script.
    /// It is stored in the key with the same name in dst.
    ///
    /// Returns 0 on success.
    pub getVariable: unsafe extern "system" fn(
        handle: *mut VSScript,
        name: *const c_char,
        dst: *mut VSMap,
    ) -> c_int,

    /// Sets all keys in the provided [`VSMap`] as variables in the script.
    ///
    /// Returns 0 on success.
    pub setVariable: unsafe extern "system" fn(
        handle: *mut VSScript,
        name: *const c_char,
        value: *const c_char,
    ) -> c_int,

    /// The returned nodes must be freed using [`freeNode()`][VSAPI::freeNode] before calling
    /// [`freeScript()`][Self::freeScript] since they may depend on data in the [`VSScript`]
    /// environment. Returns `NULL` if no node was set as output in the script.
    /// Index 0 is used by default in scripts and other values are rarely used.
    pub getOutputNode:
        unsafe extern "system" fn(handle: *mut VSScript, index: c_int) -> *mut VSNode,
    pub getOutputAlphaNode:
        unsafe extern "system" fn(handle: *mut VSScript, index: c_int) -> *mut VSNode,
    pub getAltOutputMode: unsafe extern "system" fn(handle: *mut VSScript, index: c_int) -> c_int,

    pub freeScript: unsafe extern "system" fn(handle: *mut VSScript) -> c_int,

    #[cfg(feature = "vsscript-41")]
    ///
    /// Set whether or not the working directory is temporarily changed to the same
    /// location as the script file when evaluateFile is called. Off by default.
    pub evalSetWorkingDir:
        unsafe extern "system" fn(handle: *mut VSScript, setCWD: c_int) -> c_void,
}

extern "system" {
    pub fn getVSScriptAPI(version: c_int) -> *const VSSCRIPTAPI;
}

/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

fn main() {
    #[cfg(any(feature = "link-vs", feature = "link-vsscript"))]
    {
        use std::env;

        const LIBRARY_DIR_VARIABLE: &str = "VAPOURSYNTH_LIB_PATH";

        // Make sure the build script is rerun if our env variable is changed.
        println!("cargo:rerun-if-env-changed={LIBRARY_DIR_VARIABLE}");

        if let Ok(dir) = env::var(LIBRARY_DIR_VARIABLE) {
            println!("cargo:rustc-link-search={dir}");
        }
    }
}

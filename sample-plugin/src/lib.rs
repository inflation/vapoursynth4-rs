/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

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

#[cfg(test)]
mod tests {
    use testresult::TestResult;
    use vapoursynth4_rs::sciprt::Script;

    #[test]
    fn test_vsscript_works() -> TestResult {
        let vss = Script::default();
        vss.evaluate_file(c"test.vpy")?;
        let node = vss.get_output(0)?;
        unsafe {
            (vss.get_api().freeNode)(node);
        }

        Ok(())
    }
}

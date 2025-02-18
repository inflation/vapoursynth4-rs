/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::ops::Deref;

use crate::ffi;

pub type RequestPattern = ffi::VSRequestPattern;
pub type FilterDependency = ffi::VSFilterDependency;

#[repr(transparent)]
pub struct Dependencies {
    inner: [FilterDependency],
}

impl Dependencies {
    #[must_use]
    pub fn new(deps: &[FilterDependency]) -> Option<&Dependencies> {
        i32::try_from(deps.len()).ok().map(|_| unsafe {
            &*(std::ptr::from_ref::<[FilterDependency]>(deps) as *const Dependencies)
        })
    }
}

impl<const N: usize> From<[FilterDependency; N]> for &Dependencies {
    fn from(deps: [FilterDependency; N]) -> Self {
        unsafe {
            &*(std::ptr::from_ref::<[FilterDependency]>(deps.as_slice()) as *const Dependencies)
        }
    }
}

impl Deref for Dependencies {
    type Target = [FilterDependency];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

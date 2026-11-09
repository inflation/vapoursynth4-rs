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

impl Deref for Dependencies {
    type Target = [FilterDependency];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

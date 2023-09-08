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
        i32::try_from(deps.len())
            .ok()
            .map(|_| unsafe { &*(deps as *const [FilterDependency] as *const Dependencies) })
    }
}

impl<const N: usize> From<[FilterDependency; N]> for &Dependencies {
    fn from(deps: [FilterDependency; N]) -> Self {
        unsafe { &*(deps.as_slice() as *const [FilterDependency] as *const Dependencies) }
    }
}

impl Deref for Dependencies {
    type Target = [FilterDependency];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

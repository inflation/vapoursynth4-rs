/*
 This Source Code Form is subject to the terms of the Mozilla Public
 License, v. 2.0. If a copy of the MPL was not distributed with this
 file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use std::{
    ffi::{c_char, CStr},
    fmt::{Debug, Display},
    ops::Deref,
    ptr,
};

use thiserror::Error;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(transparent)]
pub struct Key {
    inner: CStr,
}

impl Key {
    #[must_use]
    pub const fn from_cstr(str: &CStr) -> &Self {
        let mut i = 0;
        let slice = str.to_bytes();
        while i < slice.len() {
            let c = slice[i];
            assert!(
                c.is_ascii_alphanumeric() || c == b'_',
                "Key must be alphanumeric or underscore"
            );
            i += 1;
        }
        unsafe { Self::from_cstr_unchecked(str) }
    }

    const unsafe fn from_cstr_unchecked(str: &CStr) -> &Self {
        &*(ptr::from_ref(str) as *const Key)
    }

    pub(crate) unsafe fn from_ptr<'a>(ptr: *const c_char) -> &'a Self {
        Self::from_cstr(CStr::from_ptr(ptr))
    }
}

impl Deref for Key {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { f.write_str(std::str::from_utf8_unchecked(self.inner.to_bytes())) }
    }
}

#[macro_export]
macro_rules! key {
    ($s:literal) => {
        const {
            $crate::map::Key::from_cstr(unsafe {
                &CStr::from_bytes_with_nul_unchecked(concat!($s, "\0").as_bytes())
            })
        }
    };
}

#[derive(Debug, Error)]
#[error("Key is invalid. Only ascii alphanumeric or underscore is allowed.")]
pub struct InvalidKey;

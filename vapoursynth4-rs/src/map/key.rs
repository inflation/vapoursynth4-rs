use std::{
    ffi::{c_char, CStr, CString},
    fmt::{Debug, Display},
    ops::Deref, ptr,
};

use thiserror::Error;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(transparent)]
pub struct Key {
    inner: CString,
}

impl Key {
    /// # Errors
    ///
    /// Return [`InvalidKey`] if the key contains characters that are not alphanumeric
    /// or underscore
    pub fn new<T>(val: T) -> Result<Self, InvalidKey>
    where
        T: Into<Vec<u8>>,
    {
        let mut val: Vec<u8> = val.into();
        if let Some(i) = val.iter().position(|&c| c == 0) {
            val.drain(i..);
        }
        if val.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'_') {
            val.push(0);
            Ok(Self {
                inner: unsafe { CString::from_vec_with_nul_unchecked(val) },
            })
        } else {
            Err(InvalidKey)
        }
    }
}

impl Deref for Key {
    type Target = KeyStr;

    fn deref(&self) -> &Self::Target {
        // SAFETY: Key is validated
        unsafe { KeyStr::from_cstr_unchecked(self.inner.as_c_str()) } 
    }
}

impl From<&KeyStr> for Key {
    fn from(value: &KeyStr) -> Self {
        Self {
            inner: value.inner.into(),
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SAFETY: Key is validated
        unsafe { f.write_str(std::str::from_utf8_unchecked(self.inner.as_bytes())) }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(transparent)]
pub struct KeyStr {
    inner: CStr,
}

impl KeyStr {
    #[must_use]
    pub const fn from_cstr(str: &CStr) -> &Self {
        let mut i = 0;
        let slice = str.to_bytes();
        while i < slice.len() {
            let c = slice[i];
            assert!(c.is_ascii_alphanumeric() || c == b'_', "Key must be alphanumeric or underscore");
            i += 1;
        }
        unsafe { Self::from_cstr_unchecked(str) }
    }

    #[must_use]
    /// # Safety
    /// The caller must ensure that the key is valid to contain only characters that are alphanumeric or underscore
    pub const unsafe fn from_cstr_unchecked(str: &CStr) -> &Self {
        &*(ptr::from_ref(str) as *const KeyStr)
    }

    pub(crate) unsafe fn from_ptr<'a>(ptr: *const c_char) -> &'a Self {
        Self::from_cstr(CStr::from_ptr(ptr))
    }
}

impl Deref for KeyStr {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Display for KeyStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { f.write_str(std::str::from_utf8_unchecked(self.inner.to_bytes())) }
    }
}

#[macro_export]
macro_rules! key {
    ($s:expr) => {
        const { $crate::map::KeyStr::from_cstr($s) }
    };
}

#[derive(Debug, Error)]
#[error("Key is invalid. Only ascii alphanumeric or underscore is allowed.")]
pub struct InvalidKey;

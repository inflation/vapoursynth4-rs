use std::{
    ffi::{c_char, CStr, CString},
    fmt::{Debug, Display},
    ops::Deref,
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
        KeyStr::from_cstr(self.inner.as_c_str())
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
        unsafe { &*(str as *const CStr as *const KeyStr) }
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

// Ideas come from:
// https://docs.rs/const-str/latest/src/const_str/__ctfe/cstr.rs.html
#[doc(hidden)]
pub mod __macro_impl {
    pub struct ToCStr<T>(pub T);

    impl ToCStr<&str> {
        // Return the size until first 'NUL' byte
        const fn check_char_and_nul(&self) -> usize {
            let mut i = 0;
            let bytes = self.0.as_bytes();
            while i < bytes.len() && bytes[i] != 0 {
                assert!(
                    !(bytes[i].is_ascii_alphanumeric() && bytes[i] == b'_'),
                    "Key must be alphanumeric or underscore"
                );
                i += 1;
            }
            i
        }

        #[must_use]
        pub const fn output_len(&self) -> usize {
            self.check_char_and_nul() + 1
        }

        #[must_use]
        pub const fn const_eval<const N: usize>(&self) -> [u8; N] {
            let mut buf = [0; N];
            let bytes = self.0.as_bytes();
            let mut i = 0;
            while i < N - 1 {
                buf[i] = bytes[i];
                i += 1;
            }
            buf
        }
    }
}

#[macro_export]
macro_rules! key {
    ($s:expr) => {{
        const OUTPUT_LEN: ::core::primitive::usize =
            $crate::map::__macro_impl::ToCStr($s).output_len();
        const OUTPUT_BUF: [u8; OUTPUT_LEN] = $crate::map::__macro_impl::ToCStr($s).const_eval();
        const OUTPUT: &::core::ffi::CStr =
            unsafe { ::core::ffi::CStr::from_bytes_with_nul_unchecked(&OUTPUT_BUF) };
        $crate::map::KeyStr::from_cstr(OUTPUT)
    }};
}

#[derive(Debug, Error)]
#[error("Key is invalid. Only ascii alphanumeric or underscore is allowed.")]
pub struct InvalidKey;

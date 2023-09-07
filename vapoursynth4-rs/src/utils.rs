use std::ffi::CString;

pub trait ToCString {
    fn into_cstring_lossy(self) -> CString;
}

impl ToCString for String {
    fn into_cstring_lossy(mut self) -> CString {
        self.retain(|c| c != '\0');
        unsafe { CString::from_vec_unchecked(self.into_bytes()) }
    }
}

impl ToCString for &str {
    fn into_cstring_lossy(self) -> CString {
        unsafe { CString::from_vec_unchecked(self.bytes().filter(|&c| c != b'\0').collect()) }
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

use std::ffi::CString;

pub(crate) trait ToCString {
    fn to_cstring_until_nul(&self) -> CString;
}

impl ToCString for str {
    fn to_cstring_until_nul(&self) -> CString {
        unsafe {
            CString::from_vec_unchecked(
                self.as_bytes()
                    .iter()
                    .take_while(|&&c| c != b'\0')
                    .copied()
                    .collect(),
            )
        }
    }
}

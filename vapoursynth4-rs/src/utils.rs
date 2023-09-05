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
        unsafe {
            CString::from_vec_unchecked(self.bytes().into_iter().filter(|&c| c != b'\0').collect())
        }
    }
}

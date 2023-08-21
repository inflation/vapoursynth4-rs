use std::{
    ffi::{c_char, c_int, CStr, CString},
    ptr::NonNull,
};

use thiserror::Error;
use vapoursynth4_sys as ffi;

use crate::{api, FrameRef, FunctionRef, NodeRef};

pub struct Map {
    handle: NonNull<ffi::VSMap>,
}

impl Map {
    #[must_use]
    pub fn new() -> Self {
        Self {
            // safety: `api.createMap` always returns a valid pointer
            handle: unsafe { NonNull::new_unchecked((api().createMap)()) },
        }
    }

    pub(crate) unsafe fn from_raw(ptr: *mut ffi::VSMap) -> Self {
        Self {
            handle: NonNull::new_unchecked(ptr),
        }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const ffi::VSMap {
        self.handle.as_ptr()
    }

    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut ffi::VSMap {
        self.handle.as_ptr()
    }

    pub fn clear(&mut self) {
        // safety: `self.handle` is a valid pointer
        unsafe { (api().clearMap)(self.handle.as_ptr()) }
    }

    pub fn set_error(&mut self, msg: &CStr) {
        // safety: `self.handle` and `msg` are valid pointers
        unsafe { (api().mapSetError)(self.handle.as_ptr(), msg.as_ptr()) }
    }

    #[must_use]
    pub fn get_error(&self) -> Option<CString> {
        let ptr = unsafe { (api().mapGetError)(self.handle.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr).into() })
        }
    }

    #[must_use]
    pub fn len(&self) -> i32 {
        // safety: `self.handle` is a valid pointer
        unsafe { (api().mapNumKeys)(self.handle.as_ptr()) }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[must_use]
    pub fn get_key(&self, index: i32) -> &CStr {
        assert!(!(index < 0 || index >= self.len()), "index out of bounds");

        // safety: `self.handle` is a valid pointer
        unsafe { CStr::from_ptr((api().mapGetKey)(self.handle.as_ptr(), index)) }
    }

    pub fn delete_key(&mut self, key: &CStr) {
        // safety: `self.handle` and `key` are valid pointers
        unsafe { (api().mapDeleteKey)(self.handle.as_ptr(), key.as_ptr()) };
    }

    #[must_use]
    pub fn num_elements(&self, key: &CStr) -> Option<i32> {
        // safety: `self.handle` is a valid pointer
        let res = unsafe { (api().mapNumElements)(self.handle.as_ptr(), key.as_ptr()) };
        if res == -1 {
            None
        } else {
            Some(res)
        }
    }

    unsafe fn _get<T>(
        &self,
        func: unsafe extern "system" fn(
            *const ffi::VSMap,
            *const c_char,
            c_int,
            *mut ffi::VSMapPropertyError,
        ) -> T,
        key: &CStr,
        index: i32,
        error: &mut ffi::VSMapPropertyError,
    ) -> Result<T, MapPropertyError> {
        handle_get_error(func(self.as_ptr(), key.as_ptr(), index, error), *error)
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get(&self, key: &CStr, index: i32) -> Result<Value, MapPropertyError> {
        use ffi::VSPropertyType as t;

        let mut error = ffi::VSMapPropertyError::peSuccess;

        unsafe {
            match (api().mapGetType)(self.as_ptr(), key.as_ptr()) {
                t::ptUnset => Err(MapPropertyError::KeyNotFound),
                t::ptInt => {
                    let res = self._get(api().mapGetInt, key, index, &mut error)?;
                    Ok(Value::Int(res))
                }
                t::ptFloat => {
                    let res = self._get(api().mapGetFloat, key, index, &mut error)?;
                    Ok(Value::Float(res))
                }
                t::ptData => {
                    use ffi::VSDataTypeHint as dt;

                    let size = self._get(api().mapGetDataSize, key, index, &mut error)?;
                    match self._get(api().mapGetDataTypeHint, key, index, &mut error)? {
                        dt::dtUnknown | dt::dtBinary => {
                            let ptr = self._get(api().mapGetData, key, index, &mut error)?;

                            #[allow(clippy::cast_sign_loss)]
                            Ok(Value::Data(std::slice::from_raw_parts(
                                ptr.cast(),
                                size as _,
                            )))
                        }
                        dt::dtUtf8 => {
                            let ptr = self._get(api().mapGetData, key, index, &mut error)?;

                            #[allow(clippy::cast_sign_loss)]
                            Ok(Value::Utf8(std::str::from_utf8_unchecked(
                                std::slice::from_raw_parts(ptr.cast(), size as _),
                            )))
                        }
                    }
                }
                t::ptFunction => {
                    let res = self._get(api().mapGetFunction, key, index, &mut error)?;
                    Ok(Value::Function(FunctionRef::from_raw(res)))
                }
                t::ptVideoNode => {
                    let res = self._get(api().mapGetNode, key, index, &mut error)?;
                    Ok(Value::VideoNode(NodeRef::from_raw(res)))
                }
                t::ptAudioNode => {
                    let res = self._get(api().mapGetNode, key, index, &mut error)?;
                    Ok(Value::AudioNode(NodeRef::from_raw(res)))
                }
                t::ptVideoFrame => {
                    let res = self._get(api().mapGetFrame, key, index, &mut error)?;
                    Ok(Value::VideoFrame(FrameRef::from_raw(res)))
                }
                t::ptAudioFrame => {
                    let res = self._get(api().mapGetFrame, key, index, &mut error)?;
                    Ok(Value::AudioFrame(FrameRef::from_raw(res)))
                }
            }
        }
    }

    /// # Panics
    ///
    /// Panics if the key exists or is invalid
    pub fn set_empty(&mut self, key: &CStr, type_: ffi::VSPropertyType) {
        // safety: `self.handle` is a valid pointer
        let res = unsafe { (api().mapSetEmpty)(self.handle.as_ptr(), key.as_ptr(), type_) };
        assert!(res != 0);
    }

    unsafe fn _set<T>(
        &mut self,
        func: unsafe extern "system" fn(
            *mut ffi::VSMap,
            *const c_char,
            T,
            ffi::VSMapAppendMode,
        ) -> c_int,
        key: &CStr,
        val: T,
        append: ffi::VSMapAppendMode,
    ) -> Result<(), MapPropertyError> {
        handle_set_error(func(self.as_mut_ptr(), key.as_ptr(), val, append))
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError::InvalidType`] if the `key`'s type is not the same with `val`
    ///
    /// # Panics
    ///
    /// Panic if the [`Value::Data`]'s or [`Value::Utf8`]'s len is larger than [`i32::MAX`]
    pub fn set(
        &mut self,
        key: &CStr,
        val: Value,
        append: AppendMode,
    ) -> Result<(), MapPropertyError> {
        unsafe {
            match val {
                Value::Int(val) => self._set(api().mapSetInt, key, val, append.into()),
                Value::Float(val) => self._set(api().mapSetFloat, key, val, append.into()),
                Value::Data(val) => handle_set_error((api().mapSetData)(
                    self.as_mut_ptr(),
                    key.as_ptr(),
                    val.as_ptr().cast(),
                    val.len().try_into().unwrap(),
                    ffi::VSDataTypeHint::dtBinary,
                    append.into(),
                )),
                Value::Utf8(val) => handle_set_error((api().mapSetData)(
                    self.as_mut_ptr(),
                    key.as_ptr(),
                    val.as_ptr().cast(),
                    val.len().try_into().unwrap(),
                    ffi::VSDataTypeHint::dtUtf8,
                    append.into(),
                )),
                Value::VideoNode(val) | Value::AudioNode(val) => {
                    self._set(api().mapSetNode, key, val.as_mut_ptr(), append.into())
                }
                Value::VideoFrame(val) | Value::AudioFrame(val) => {
                    self._set(api().mapSetFrame, key, val.as_mut_ptr(), append.into())
                }
                Value::Function(val) => {
                    self._set(api().mapSetFunction, key, val.as_mut_ptr(), append.into())
                }
            }
        }
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Map {
    fn drop(&mut self) {
        // safety: `self.handle` is a valid pointer
        unsafe { (api().freeMap)(self.handle.as_ptr()) }
    }
}

impl Clone for Map {
    fn clone(&self) -> Self {
        let mut map = Self::new();
        // safety: `self` and `map` are both valid
        unsafe { (api().copyMap)(self.as_ptr(), map.as_mut_ptr()) };
        map
    }
}

fn handle_get_error<T>(res: T, error: ffi::VSMapPropertyError) -> Result<T, MapPropertyError> {
    use ffi::VSMapPropertyError as e;
    use MapPropertyError as pe;

    match error {
        e::peSuccess => Ok(res),
        e::peUnset => Err(pe::KeyNotFound),
        e::peType => Err(pe::InvalidType),
        e::peIndex => Err(pe::IndexOutOfBound),
        e::peError => Err(pe::MapError),
    }
}

fn handle_set_error(res: i32) -> Result<(), MapPropertyError> {
    if res == 0 {
        Ok(())
    } else {
        Err(MapPropertyError::InvalidType)
    }
}

pub enum Value<'m> {
    Int(i64),
    Float(f64),
    /// Arbitrary binary data
    ///
    /// # Notes
    ///
    /// Could still be UTF-8 strings because of the API3 compatibility
    Data(&'m [u8]),
    Utf8(&'m str),
    VideoNode(NodeRef),
    AudioNode(NodeRef),
    VideoFrame(FrameRef),
    AudioFrame(FrameRef),
    Function(FunctionRef),
}

#[derive(Debug, Error)]
pub enum MapPropertyError {
    #[error("The requested key was not found in the map")]
    KeyNotFound,
    #[error("The wrong function was used to retrieve the property")]
    InvalidType,
    #[error("The requested index was out of bound")]
    IndexOutOfBound,
    #[error("The map has the error state set")]
    MapError,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum AppendMode {
    Replace,
    Append,
}

impl From<ffi::VSMapAppendMode> for AppendMode {
    fn from(value: ffi::VSMapAppendMode) -> Self {
        match value {
            ffi::VSMapAppendMode::maReplace => AppendMode::Replace,
            ffi::VSMapAppendMode::maAppend => AppendMode::Append,
        }
    }
}

impl From<AppendMode> for ffi::VSMapAppendMode {
    fn from(value: AppendMode) -> Self {
        match value {
            AppendMode::Replace => ffi::VSMapAppendMode::maReplace,
            AppendMode::Append => ffi::VSMapAppendMode::maAppend,
        }
    }
}

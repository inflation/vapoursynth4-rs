use std::{
    ffi::{c_char, c_int, CStr},
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use thiserror::Error;

use crate::{
    api, ffi,
    frame::{AudioFrame, Frame, VideoFrame},
    function::Function,
    node::{AudioNode, Node, VideoNode},
};

mod key;
pub use key::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MapMut<'m> {
    handle: NonNull<ffi::VSMap>,
    _marker: PhantomData<&'m mut Map>,
}

impl<'m> MapMut<'m> {
    #[must_use]
    pub fn new(handle: NonNull<ffi::VSMap>) -> Self {
        Self {
            handle,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSMap) -> MapMut<'m> {
        MapMut {
            handle: NonNull::new_unchecked(ptr),
            _marker: PhantomData,
        }
    }
}

impl<'m> Deref for MapMut<'m> {
    type Target = Map;

    fn deref(&self) -> &'m Self::Target {
        unsafe { &*(self as *const Self).cast::<Map>() }
    }
}

impl<'m> DerefMut for MapMut<'m> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self as *mut Self).cast::<Map>() }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
pub struct MapRef<'m> {
    handle: NonNull<ffi::VSMap>,
    _marker: PhantomData<&'m Map>,
}

impl<'m> MapRef<'m> {
    #[must_use]
    pub fn new(handle: NonNull<ffi::VSMap>) -> Self {
        Self {
            handle,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub unsafe fn from_ptr(ptr: *const ffi::VSMap) -> MapRef<'m> {
        MapRef {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
            _marker: PhantomData,
        }
    }
}

impl<'m> Deref for MapRef<'m> {
    type Target = Map;

    fn deref(&self) -> &'m Self::Target {
        unsafe { &*(self as *const Self).cast::<Map>() }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
#[repr(transparent)]
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

    #[must_use]
    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSMap) -> Self {
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
        unsafe { (api().clearMap)(self.as_mut_ptr()) }
    }

    pub fn set_error(&mut self, msg: &CStr) {
        // safety: `self.handle` and `msg` are valid pointers
        unsafe { (api().mapSetError)(self.as_mut_ptr(), msg.as_ptr()) }
    }

    #[must_use]
    pub fn get_error(&self) -> Option<&CStr> {
        let ptr = unsafe { (api().mapGetError)(self.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }

    #[must_use]
    pub fn len(&self) -> i32 {
        // safety: `self.handle` is a valid pointer
        unsafe { (api().mapNumKeys)(self.as_ptr()) }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[must_use]
    pub fn get_key(&self, index: i32) -> &KeyStr {
        assert!(!(index < 0 || index >= self.len()), "index out of bounds");

        // safety: `self.handle` is a valid pointer
        unsafe { KeyStr::from_ptr((api().mapGetKey)(self.as_ptr(), index)) }
    }

    pub fn delete_key(&mut self, key: &KeyStr) {
        // safety: `self.handle` and `key` are valid pointers
        unsafe { (api().mapDeleteKey)(self.as_mut_ptr(), key.as_ptr()) };
    }

    #[must_use]
    pub fn num_elements(&self, key: &KeyStr) -> Option<i32> {
        // safety: `self.handle` is a valid pointer
        let res = unsafe { (api().mapNumElements)(self.as_ptr(), key.as_ptr()) };
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
        key: &KeyStr,
        index: i32,
    ) -> Result<T, MapPropertyError> {
        let mut error = ffi::VSMapPropertyError::Success;
        handle_get_error(func(self.as_ptr(), key.as_ptr(), index, &mut error), error)
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_int(&self, key: &KeyStr, index: i32) -> Result<i64, MapPropertyError> {
        unsafe { self._get(api().mapGetInt, key, index) }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_float(&self, key: &KeyStr, index: i32) -> Result<f64, MapPropertyError> {
        unsafe { self._get(api().mapGetFloat, key, index) }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    #[allow(clippy::cast_sign_loss)]
    pub fn get_binary(&self, key: &KeyStr, index: i32) -> Result<&[u8], MapPropertyError> {
        use ffi::VSDataTypeHint as dt;

        unsafe {
            if let dt::Unknown | dt::Binary = self._get(api().mapGetDataTypeHint, key, index)? {
                let size = self._get(api().mapGetDataSize, key, index)?;
                let ptr = self._get(api().mapGetData, key, index)?;

                Ok(std::slice::from_raw_parts(ptr.cast(), size as _))
            } else {
                Err(MapPropertyError::InvalidType)
            }
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    #[allow(clippy::cast_sign_loss)]
    pub fn get_utf8(&self, key: &KeyStr, index: i32) -> Result<&str, MapPropertyError> {
        unsafe {
            if let ffi::VSDataTypeHint::Utf8 = self._get(api().mapGetDataTypeHint, key, index)? {
                let size = self._get(api().mapGetDataSize, key, index)?;
                let ptr = self._get(api().mapGetData, key, index)?;

                Ok(std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    ptr.cast(),
                    size as _,
                )))
            } else {
                Err(MapPropertyError::InvalidType)
            }
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_function(&self, key: &KeyStr, index: i32) -> Result<Function, MapPropertyError> {
        unsafe {
            self._get(api().mapGetFunction, key, index)
                .map(|p| Function::from_ptr(p))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_video_node(&self, key: &KeyStr, index: i32) -> Result<VideoNode, MapPropertyError> {
        unsafe {
            self._get(api().mapGetNode, key, index)
                .map(|p| VideoNode::from_ptr(p))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_audio_node(&self, key: &KeyStr, index: i32) -> Result<AudioNode, MapPropertyError> {
        unsafe {
            self._get(api().mapGetNode, key, index)
                .map(|p| AudioNode::from_ptr(p))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_video_frame(
        &self,
        key: &KeyStr,
        index: i32,
    ) -> Result<VideoFrame, MapPropertyError> {
        unsafe {
            self._get(api().mapGetFrame, key, index)
                .map(|p| VideoFrame::from_ptr(p))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_audio_frame(
        &self,
        key: &KeyStr,
        index: i32,
    ) -> Result<AudioFrame, MapPropertyError> {
        unsafe {
            self._get(api().mapGetFrame, key, index)
                .map(|p| AudioFrame::from_ptr(p))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get(&self, key: &KeyStr, index: i32) -> Result<Value, MapPropertyError> {
        use ffi::VSPropertyType as t;

        unsafe {
            match (api().mapGetType)(self.as_ptr(), key.as_ptr()) {
                t::Unset => Err(MapPropertyError::KeyNotFound),
                t::Int => self.get_int(key, index).map(Value::Int),
                t::Float => self.get_float(key, index).map(Value::Float),
                t::Data => {
                    use ffi::VSDataTypeHint as dt;

                    let size = self._get(api().mapGetDataSize, key, index)?;
                    #[allow(clippy::cast_sign_loss)]
                    match self._get(api().mapGetDataTypeHint, key, index)? {
                        dt::Unknown | dt::Binary => {
                            let ptr = self._get(api().mapGetData, key, index)?;
                            Ok(Value::Data(std::slice::from_raw_parts(
                                ptr.cast(),
                                size as _,
                            )))
                        }
                        dt::Utf8 => {
                            let ptr = self._get(api().mapGetData, key, index)?;
                            Ok(Value::Utf8(std::str::from_utf8_unchecked(
                                std::slice::from_raw_parts(ptr.cast(), size as _),
                            )))
                        }
                    }
                }
                t::Function => self.get_function(key, index).map(Value::Function),
                t::VideoNode => self.get_video_node(key, index).map(Value::VideoNode),
                t::AudioNode => self.get_audio_node(key, index).map(Value::AudioNode),
                t::VideoFrame => self.get_video_frame(key, index).map(Value::VideoFrame),
                t::AudioFrame => self.get_audio_frame(key, index).map(Value::AudioFrame),
            }
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_int_saturated(&self, key: &KeyStr, index: i32) -> Result<i32, MapPropertyError> {
        unsafe { self._get(api().mapGetIntSaturated, key, index) }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_int_array(&self, key: &KeyStr) -> Result<&[i64], MapPropertyError> {
        let mut error = ffi::VSMapPropertyError::Success;
        unsafe {
            let size = self
                .num_elements(key)
                .ok_or(MapPropertyError::KeyNotFound)?;
            let ptr = handle_get_error(
                (api().mapGetIntArray)(self.as_ptr(), key.as_ptr(), &mut error),
                error,
            )?;

            #[allow(clippy::cast_sign_loss)]
            Ok(std::slice::from_raw_parts(ptr, size as _))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_float_saturated(&self, key: &KeyStr, index: i32) -> Result<f32, MapPropertyError> {
        // safety: `self.handle` is a valid pointer
        unsafe { self._get(api().mapGetFloatSaturated, key, index) }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_float_array(&self, key: &KeyStr) -> Result<&[f64], MapPropertyError> {
        let mut error = ffi::VSMapPropertyError::Success;
        unsafe {
            let size = self
                .num_elements(key)
                .ok_or(MapPropertyError::KeyNotFound)?;
            let ptr = handle_get_error(
                (api().mapGetFloatArray)(self.as_ptr(), key.as_ptr(), &mut error),
                error,
            )?;

            #[allow(clippy::cast_sign_loss)]
            Ok(std::slice::from_raw_parts(ptr, size as _))
        }
    }

    /// # Panics
    ///
    /// Panics if the key exists or is invalid
    pub fn set_empty(&mut self, key: &KeyStr, type_: ffi::VSPropertyType) {
        // safety: `self.handle` is a valid pointer
        let res = unsafe { (api().mapSetEmpty)(self.as_mut_ptr(), key.as_ptr(), type_) };
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
        key: &KeyStr,
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
        key: &KeyStr,
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
                    ffi::VSDataTypeHint::Binary,
                    append.into(),
                )),
                Value::Utf8(val) => handle_set_error((api().mapSetData)(
                    self.as_mut_ptr(),
                    key.as_ptr(),
                    val.as_ptr().cast(),
                    val.len().try_into().unwrap(),
                    ffi::VSDataTypeHint::Utf8,
                    append.into(),
                )),
                Value::VideoNode(val) => self._set(
                    api().mapSetNode,
                    key,
                    val.as_ptr().cast_mut(),
                    append.into(),
                ),
                Value::AudioNode(val) => self._set(
                    api().mapSetNode,
                    key,
                    val.as_ptr().cast_mut(),
                    append.into(),
                ),
                Value::VideoFrame(val) => {
                    self._set(api().mapSetFrame, key, val.as_ptr(), append.into())
                }
                Value::AudioFrame(val) => {
                    self._set(api().mapSetFrame, key, val.as_ptr(), append.into())
                }
                Value::Function(val) => {
                    self._set(api().mapSetFunction, key, val.as_ptr(), append.into())
                }
            }
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    ///
    /// # Panics
    ///
    /// Panic if the `val.len()` is larger than [`i32::MAX`]
    pub fn set_int_array(&mut self, key: &KeyStr, val: &[i64]) -> Result<(), MapPropertyError> {
        unsafe {
            handle_set_error((api().mapSetIntArray)(
                self.as_mut_ptr(),
                key.as_ptr(),
                val.as_ptr(),
                val.len().try_into().unwrap(),
            ))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    ///
    /// # Panics
    ///
    /// Panic if the `val.len()` is larger than [`i32::MAX`]
    pub fn set_float_array(&mut self, key: &KeyStr, val: &[f64]) -> Result<(), MapPropertyError> {
        unsafe {
            handle_set_error((api().mapSetFloatArray)(
                self.as_mut_ptr(),
                key.as_ptr(),
                val.as_ptr(),
                val.len().try_into().unwrap(),
            ))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn consume_node(
        &mut self,
        key: &KeyStr,
        node: impl Node,
        append: AppendMode,
    ) -> Result<(), MapPropertyError> {
        let mut node = ManuallyDrop::new(node);
        unsafe {
            handle_set_error((api().mapConsumeNode)(
                self.as_mut_ptr(),
                key.as_ptr(),
                node.as_mut_ptr(),
                append.into(),
            ))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn consume_frame(
        &mut self,
        key: &KeyStr,
        frame: impl Frame,
        append: AppendMode,
    ) -> Result<(), MapPropertyError> {
        let frame = ManuallyDrop::new(frame);
        unsafe {
            handle_set_error((api().mapConsumeFrame)(
                self.as_mut_ptr(),
                key.as_ptr(),
                frame.as_ptr(),
                append.into(),
            ))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn consume_function(
        &mut self,
        key: &KeyStr,
        function: Function,
        append: AppendMode,
    ) -> Result<(), MapPropertyError> {
        let function = ManuallyDrop::new(function);
        unsafe {
            handle_set_error((api().mapConsumeFunction)(
                self.as_mut_ptr(),
                key.as_ptr(),
                function.as_ptr(),
                append.into(),
            ))
        }
    }
}

impl Drop for Map {
    fn drop(&mut self) {
        // safety: `self.handle` is a valid pointer
        unsafe { (api().freeMap)(self.as_mut_ptr()) }
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

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

fn handle_get_error<T>(res: T, error: ffi::VSMapPropertyError) -> Result<T, MapPropertyError> {
    use ffi::VSMapPropertyError as e;
    use MapPropertyError as pe;

    match error {
        e::Success => Ok(res),
        e::Unset => Err(pe::KeyNotFound),
        e::Type => Err(pe::InvalidType),
        e::Index => Err(pe::IndexOutOfBound),
        e::Error => Err(pe::MapError),
    }
}

fn handle_set_error(res: i32) -> Result<(), MapPropertyError> {
    if res == 0 {
        Ok(())
    } else {
        Err(MapPropertyError::InvalidType)
    }
}

#[derive(Clone, Debug)]
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
    VideoNode(VideoNode),
    AudioNode(AudioNode),
    VideoFrame(VideoFrame),
    AudioFrame(AudioFrame),
    Function(Function),
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Error)]
pub enum MapPropertyError {
    #[error("The requested key was not found in the map")]
    KeyNotFound,
    #[error("The wrong function was used to retrieve the property")]
    InvalidType,
    #[error("The requested index was out of bound")]
    IndexOutOfBound,
    #[error("The map has errors. Use [`Map::get_error`] to retrieve the message")]
    MapError,
}

pub type AppendMode = ffi::VSMapAppendMode;

#[cfg(test)]
mod tests {
    use core::panic;

    use const_str::cstr;
    use testresult::TestResult;

    use crate::set_api_default;

    use super::*;

    #[test]
    fn clear() -> TestResult {
        set_api_default()?;
        let mut map = Map::default();
        let key = crate::key!("what");
        map.set(key, Value::Int(42), AppendMode::Replace)?;

        map.clear();
        match map.get(key, 0) {
            Err(MapPropertyError::KeyNotFound) => Ok(()),
            _ => panic!("Map is not cleared"),
        }
    }

    #[test]
    fn error() -> TestResult {
        set_api_default()?;
        let mut map = Map::default();
        let key = crate::key!("what");
        map.set(key, Value::Float(42.0), AppendMode::Replace)?;

        map.set_error(cstr!("Yes"));
        match map.get_error() {
            Some(msg) => assert_eq!(msg, cstr!("Yes"), "Error message is not match"),
            None => panic!("Error is not set"),
        }
        let res = map.get(key, 0);
        match res {
            Err(MapPropertyError::KeyNotFound) => {}
            _ => panic!("Map is not cleared after setting error"),
        }

        map.set(key, Value::Float(42.0), AppendMode::Replace)?;
        let res = map.get(key, 0);
        match res {
            Err(MapPropertyError::MapError) => {}
            _ => panic!(
                "Map after setting error can only be freed, \
                cleared, or queried for error"
            ),
        }

        Ok(())
    }

    #[test]
    fn len() -> TestResult {
        set_api_default()?;
        let mut map = Map::default();
        let key = crate::key!("what");

        map.set(key, Value::Data(&[42, 43, 44, 45]), AppendMode::Replace)?;
        assert_eq!(1, map.len(), "Number of keys is not correct");

        assert!(!map.is_empty(), "Map is not empty");

        Ok(())
    }

    #[test]
    fn key() -> TestResult {
        set_api_default()?;
        let mut map = Map::default();
        let key = crate::key!("what");

        map.set(key, Value::Float(42.0), AppendMode::Append)?;

        assert_eq!(key, map.get_key(0), "Key is not correct");

        match map.num_elements(key) {
            Some(num) => assert_eq!(1, num),
            None => panic!("Key `{key}` not found "),
        }

        map.delete_key(key);
        assert_eq!(
            0,
            map.len(),
            "Number of keys is not correct after deleting `{key}`"
        );

        Ok(())
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn get_set() -> TestResult {
        set_api_default()?;
        let mut map = Map::default();
        let key = crate::key!("what");

        let source = i64::from(i32::MAX) + 1;
        map.set(key, Value::Int(source), AppendMode::Replace)?;
        let res = map.get(key, 0)?;
        match res {
            Value::Int(val) => assert_eq!(val, source, "Value of `{key}` is not correct"),
            _ => panic!("Invalid type of `{key}`"),
        }
        let res = map.get_int_saturated(key, 0)?;
        assert_eq!(res, i32::MAX, "Value of `{key}` is not correct");
        map.set(key, Value::Int(source), AppendMode::Append)?;
        assert_eq!(&[source, source], map.get_int_array(key)?);
        map.set_int_array(key, &[1, 2, 3])?;
        assert_eq!(&[1, 2, 3], map.get_int_array(key)?);

        map.set(key, Value::Float(f64::MAX), AppendMode::Replace)?;
        let res = map.get(key, 0)?;
        match res {
            Value::Float(val) => {
                assert_eq!(val, f64::MAX, "Value of `{key}` is not correct");
            }
            _ => panic!("Invalid type of `{key}`"),
        }
        let res = map.get_float_saturated(key, 0)?;
        assert_eq!(res, f32::MAX, "Value of `{key}` is not correct");
        map.set(key, Value::Float(f64::MAX), AppendMode::Append)?;
        assert_eq!(&[f64::MAX, f64::MAX], map.get_float_array(key)?);
        map.set_float_array(key, &[1.0, 2.0, 3.0])?;
        assert_eq!(&[1.0, 2.0, 3.0], map.get_float_array(key)?);

        map.set(key, Value::Data(&[42, 43]), AppendMode::Replace)?;
        let res = map.get(key, 0)?;
        match res {
            Value::Data(val) => {
                assert_eq!(val, &[42, 43], "Value of `{key}` is not correct");
            }
            _ => panic!("Invalid type of `{key}`"),
        }

        map.set(key, Value::Utf8("good"), AppendMode::Replace)?;
        let res = map.get(key, 0)?;
        match res {
            Value::Utf8(val) => {
                assert_eq!(val, "good", "Value of `{key}` is not correct");
            }
            _ => panic!("Invalid type of `{key}`"),
        }

        Ok(())
    }
}

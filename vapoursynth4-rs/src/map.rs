use std::{
    ffi::{c_char, c_int, CStr},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use thiserror::Error;

use crate::{
    api::Api,
    ffi,
    frame::{AudioFrame, Frame, FrameType, VideoFrame},
    function::Function,
    node::{AudioNode, Node, VideoNode},
};

mod key;
pub use key::*;

// MARK: MapRef

/// A borrowed reference to a [`ffi::VSMap`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapRef<'m> {
    handle: NonNull<ffi::VSMap>,
    api: Api,
    marker: std::marker::PhantomData<&'m ()>,
}

impl MapRef<'_> {
    // Safety: `ptr` must be valid
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: *const ffi::VSMap, api: Api) -> Self {
        debug_assert!(!ptr.is_null());
        Self {
            handle: NonNull::new_unchecked(ptr.cast_mut()),
            api,
            marker: std::marker::PhantomData,
        }
    }

    /// Returns a raw pointer to the wrapped value.
    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut ffi::VSMap {
        self.handle.as_ptr()
    }
}

impl Deref for MapRef<'_> {
    type Target = Map;

    fn deref(&self) -> &Self::Target {
        unsafe { &*std::ptr::from_ref(self).cast() }
    }
}

impl DerefMut for MapRef<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *std::ptr::from_mut(self).cast() }
    }
}

// MARK: Map

/// An owned [`ffi::VSMap`].
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Map {
    handle: NonNull<ffi::VSMap>,
    api: Api,
}

impl Map {
    // Safety: `ptr` must be a valid, owned instance created by `api`.
    #[must_use]
    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSMap, api: Api) -> Self {
        debug_assert!(!ptr.is_null());
        Self {
            handle: NonNull::new_unchecked(ptr),
            api,
        }
    }

    /// Returns a raw pointer to the wrapped value.
    #[must_use]
    pub fn as_ptr(&self) -> *mut ffi::VSMap {
        self.handle.as_ptr()
    }
}

impl Map {
    pub fn clear(&self) {
        // safety: `self.handle` is a valid pointer
        unsafe { (self.api.clearMap)(self.as_ptr()) }
    }

    pub fn set_error(&self, msg: &CStr) {
        // safety: `self.handle` and `msg` are valid pointers
        unsafe { (self.api.mapSetError)(self.as_ptr(), msg.as_ptr()) }
    }

    #[must_use]
    pub fn get_error(&self) -> Option<&CStr> {
        let ptr = unsafe { (self.api.mapGetError)(self.as_ptr()) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }

    #[must_use]
    pub fn len(&self) -> i32 {
        // safety: `self.handle` is a valid pointer
        unsafe { (self.api.mapNumKeys)(self.as_ptr()) }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // MARK: Get

    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    #[must_use]
    pub fn get_key(&self, index: i32) -> &KeyStr {
        assert!(!(index < 0 || index >= self.len()), "index out of bounds");

        // safety: `self.handle` is a valid pointer
        unsafe { KeyStr::from_ptr((self.api.mapGetKey)(self.as_ptr(), index)) }
    }

    pub fn delete_key(&mut self, key: &KeyStr) {
        // safety: `self.handle` and `key` are valid pointers
        unsafe { (self.api.mapDeleteKey)(self.as_ptr(), key.as_ptr()) };
    }

    /// # Errors
    ///
    /// Returns [`MapPropertyError`] if the key is not found.
    pub fn num_elements(&self, key: &KeyStr) -> Result<i32, MapPropertyError> {
        // safety: `self.handle` is a valid pointer
        let res = unsafe { (self.api.mapNumElements)(self.as_ptr(), key.as_ptr()) };
        if res == -1 {
            Err(MapPropertyError::KeyNotFound)
        } else {
            Ok(res)
        }
    }

    unsafe fn get_internal<T>(
        &self,
        func: unsafe extern "system-unwind" fn(
            *const ffi::VSMap,
            *const c_char,
            c_int,
            *mut ffi::VSMapPropertyError,
        ) -> T,
        key: &KeyStr,
        index: i32,
    ) -> Result<T, MapPropertyError> {
        let mut error = ffi::VSMapPropertyError::Success;
        self.handle_get_error(func(self.as_ptr(), key.as_ptr(), index, &mut error), error)
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_int(&self, key: &KeyStr, index: i32) -> Result<i64, MapPropertyError> {
        unsafe { self.get_internal(self.api.mapGetInt, key, index) }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_float(&self, key: &KeyStr, index: i32) -> Result<f64, MapPropertyError> {
        unsafe { self.get_internal(self.api.mapGetFloat, key, index) }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    #[allow(clippy::cast_sign_loss)]
    pub fn get_binary(&self, key: &KeyStr, index: i32) -> Result<&[u8], MapPropertyError> {
        use ffi::VSDataTypeHint as dt;

        unsafe {
            if let dt::Unknown | dt::Binary =
                self.get_internal(self.api.mapGetDataTypeHint, key, index)?
            {
                let size = self.get_internal(self.api.mapGetDataSize, key, index)?;
                let ptr = self.get_internal(self.api.mapGetData, key, index)?;

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
            if let ffi::VSDataTypeHint::Utf8 =
                self.get_internal(self.api.mapGetDataTypeHint, key, index)?
            {
                let size = self.get_internal(self.api.mapGetDataSize, key, index)?;
                let ptr = self.get_internal(self.api.mapGetData, key, index)?;

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
            self.get_internal(self.api.mapGetFunction, key, index)
                .map(|p| Function::from_ptr(p, self.api))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_video_node(&self, key: &KeyStr, index: i32) -> Result<VideoNode, MapPropertyError> {
        unsafe {
            self.get_internal(self.api.mapGetNode, key, index)
                .map(|p| Node::from_ptr(p, self.api))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_audio_node(&self, key: &KeyStr, index: i32) -> Result<AudioNode, MapPropertyError> {
        unsafe {
            self.get_internal(self.api.mapGetNode, key, index)
                .map(|p| Node::from_ptr(p, self.api))
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
            self.get_internal(self.api.mapGetFrame, key, index)
                .map(|p| VideoFrame::from_ptr(p, self.api))
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
            self.get_internal(self.api.mapGetFrame, key, index)
                .map(|p| AudioFrame::from_ptr(p, self.api))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get(&self, key: &KeyStr, index: i32) -> Result<Value, MapPropertyError> {
        use ffi::VSPropertyType as t;

        unsafe {
            match (self.api.mapGetType)(self.as_ptr(), key.as_ptr()) {
                t::Unset => Err(MapPropertyError::KeyNotFound),
                t::Int => self.get_int(key, index).map(Value::Int),
                t::Float => self.get_float(key, index).map(Value::Float),
                t::Data => {
                    use ffi::VSDataTypeHint as dt;

                    let size = self.get_internal(self.api.mapGetDataSize, key, index)?;
                    #[allow(clippy::cast_sign_loss)]
                    match self.get_internal(self.api.mapGetDataTypeHint, key, index)? {
                        dt::Unknown | dt::Binary => {
                            let ptr = self.get_internal(self.api.mapGetData, key, index)?;
                            Ok(Value::Data(std::slice::from_raw_parts(
                                ptr.cast(),
                                size as _,
                            )))
                        }
                        dt::Utf8 => {
                            let ptr = self.get_internal(self.api.mapGetData, key, index)?;
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
        unsafe { self.get_internal(self.api.mapGetIntSaturated, key, index) }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_int_array(&self, key: &KeyStr) -> Result<&[i64], MapPropertyError> {
        let mut error = ffi::VSMapPropertyError::Success;
        unsafe {
            let size = self.num_elements(key)?;
            let ptr = self.handle_get_error(
                (self.api.mapGetIntArray)(self.as_ptr(), key.as_ptr(), &mut error),
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
        unsafe { self.get_internal(self.api.mapGetFloatSaturated, key, index) }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn get_float_array(&self, key: &KeyStr) -> Result<&[f64], MapPropertyError> {
        let mut error = ffi::VSMapPropertyError::Success;
        unsafe {
            let size = self.num_elements(key)?;
            let ptr = self.handle_get_error(
                (self.api.mapGetFloatArray)(self.as_ptr(), key.as_ptr(), &mut error),
                error,
            )?;

            #[allow(clippy::cast_sign_loss)]
            Ok(std::slice::from_raw_parts(ptr, size as _))
        }
    }

    // MARK: Set

    /// # Panics
    ///
    /// Panics if the key exists or is invalid
    pub fn set_empty(&mut self, key: &KeyStr, type_: ffi::VSPropertyType) {
        // safety: `self.handle` is a valid pointer
        let res = unsafe { (self.api.mapSetEmpty)(self.as_ptr(), key.as_ptr(), type_) };
        assert!(res != 0);
    }

    unsafe fn set_internal<T>(
        &mut self,
        func: unsafe extern "system-unwind" fn(
            *mut ffi::VSMap,
            *const c_char,
            T,
            ffi::VSMapAppendMode,
        ) -> c_int,
        key: &KeyStr,
        val: T,
        append: ffi::VSMapAppendMode,
    ) -> Result<(), MapPropertyError> {
        handle_set_error(func(self.as_ptr(), key.as_ptr(), val, append))
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
                Value::Int(val) => self.set_internal(self.api.mapSetInt, key, val, append),
                Value::Float(val) => self.set_internal(self.api.mapSetFloat, key, val, append),
                Value::Data(val) => handle_set_error((self.api.mapSetData)(
                    self.as_ptr(),
                    key.as_ptr(),
                    val.as_ptr().cast(),
                    val.len().try_into().unwrap(),
                    ffi::VSDataTypeHint::Binary,
                    append,
                )),
                Value::Utf8(val) => handle_set_error((self.api.mapSetData)(
                    self.as_ptr(),
                    key.as_ptr(),
                    val.as_ptr().cast(),
                    val.len().try_into().unwrap(),
                    ffi::VSDataTypeHint::Utf8,
                    append,
                )),
                Value::VideoNode(val) | Value::AudioNode(val) => {
                    self.set_internal(self.api.mapSetNode, key, val.as_ptr(), append)
                }
                Value::VideoFrame(val) => {
                    self.set_internal(self.api.mapSetFrame, key, val.as_ptr(), append)
                }
                Value::AudioFrame(val) => {
                    self.set_internal(self.api.mapSetFrame, key, val.as_ptr(), append)
                }
                Value::Function(val) => {
                    self.set_internal(self.api.mapSetFunction, key, val.as_ptr(), append)
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
            handle_set_error((self.api.mapSetIntArray)(
                self.as_ptr(),
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
            handle_set_error((self.api.mapSetFloatArray)(
                self.as_ptr(),
                key.as_ptr(),
                val.as_ptr(),
                val.len().try_into().unwrap(),
            ))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn consume_node<T: FrameType>(
        &self,
        key: &KeyStr,
        node: Node<T>,
        append: AppendMode,
    ) -> Result<(), MapPropertyError> {
        let node = ManuallyDrop::new(node);
        unsafe {
            handle_set_error((self.api.mapConsumeNode)(
                self.as_ptr(),
                key.as_ptr(),
                node.as_ptr(),
                append,
            ))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn consume_frame<T: FrameType>(
        &self,
        key: &KeyStr,
        frame: Frame<T>,
        append: AppendMode,
    ) -> Result<(), MapPropertyError> {
        let frame = ManuallyDrop::new(frame);
        unsafe {
            handle_set_error((self.api.mapConsumeFrame)(
                self.as_ptr(),
                key.as_ptr(),
                frame.as_ptr(),
                append,
            ))
        }
    }

    /// # Errors
    ///
    /// Return [`MapPropertyError`] if the underlying API does not success
    pub fn consume_function(
        &self,
        key: &KeyStr,
        function: Function,
        append: AppendMode,
    ) -> Result<(), MapPropertyError> {
        let function = ManuallyDrop::new(function);
        unsafe {
            handle_set_error((self.api.mapConsumeFunction)(
                self.as_ptr(),
                key.as_ptr(),
                function.as_ptr(),
                append,
            ))
        }
    }

    fn handle_get_error<T>(
        &self,
        res: T,
        error: ffi::VSMapPropertyError,
    ) -> Result<T, MapPropertyError> {
        use ffi::VSMapPropertyError as e;
        use MapPropertyError as pe;

        match error {
            e::Success => Ok(res),
            e::Unset => Err(pe::KeyNotFound),
            e::Type => Err(pe::InvalidType),
            e::Index => Err(pe::IndexOutOfBound),
            e::Error => {
                let error = unsafe { self.get_error().unwrap_unchecked() }.to_string_lossy();
                Err(pe::MapError(error.into()))
            }
        }
    }
}

impl Drop for Map {
    fn drop(&mut self) {
        // safety: `self.handle` is a valid pointer
        unsafe { (self.api.freeMap)(self.as_ptr()) }
    }
}

impl Clone for Map {
    fn clone(&self) -> Self {
        // safety: `self` and `map` are both valid
        unsafe {
            let ptr = (self.api.createMap)();
            (self.api.copyMap)(self.as_ptr(), ptr);
            Self::from_ptr(ptr, self.api)
        }
    }
}

#[cfg(feature = "link-library")]
impl Default for Map {
    fn default() -> Self {
        unsafe {
            let api = Api::default();
            let ptr = (api.createMap)();
            Self::from_ptr(ptr, api)
        }
    }
}

unsafe impl Send for MapRef<'_> {}
unsafe impl Sync for MapRef<'_> {}

// MARK: Helper

fn handle_set_error(res: i32) -> Result<(), MapPropertyError> {
    if res == 0 {
        Ok(())
    } else {
        Err(MapPropertyError::InvalidType)
    }
}

// MARK: Types

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

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Error)]
pub enum MapPropertyError {
    #[error("The requested key was not found in the map")]
    KeyNotFound,
    #[error("The wrong function was used to retrieve the property")]
    InvalidType,
    #[error("The requested index was out of bound")]
    IndexOutOfBound,
    #[error("Error: {0}")]
    MapError(String),
}

impl From<MapPropertyError> for &'static CStr {
    fn from(value: MapPropertyError) -> Self {
        match value {
            MapPropertyError::KeyNotFound => c"KeyNotFound",
            MapPropertyError::InvalidType => c"InvalidType",
            MapPropertyError::IndexOutOfBound => c"IndexOutOfBound",
            MapPropertyError::MapError(_) => c"MapError",
        }
    }
}

pub type AppendMode = ffi::VSMapAppendMode;

// MARK: Tests

#[cfg(test)]
#[cfg(feature = "link-library")]
mod tests {
    use core::panic;

    use const_str::cstr;
    use testresult::TestResult;

    use super::*;

    #[test]
    fn clear() -> TestResult {
        let mut map = Map::default();
        let key = crate::key!(c"what");
        map.set(key, Value::Int(42), AppendMode::Replace)?;

        map.clear();
        match map.get(key, 0) {
            Err(MapPropertyError::KeyNotFound) => Ok(()),
            _ => panic!("Map is not cleared"),
        }
    }

    #[test]
    fn error() -> TestResult {
        let mut map = Map::default();
        let key = crate::key!(c"what");
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
            Err(MapPropertyError::MapError(..)) => {}
            _ => panic!(
                "Map after setting error can only be freed, \
                cleared, or queried for error"
            ),
        }

        Ok(())
    }

    #[test]
    fn len() -> TestResult {
        let mut map = Map::default();
        let key = crate::key!(c"what");

        map.set(key, Value::Data(&[42, 43, 44, 45]), AppendMode::Replace)?;
        assert_eq!(1, map.len(), "Number of keys is not correct");

        assert!(!map.is_empty(), "Map is not empty");

        Ok(())
    }

    #[test]
    fn key() -> TestResult {
        let mut map = Map::default();
        let key = crate::key!(c"what");

        map.set(key, Value::Float(42.0), AppendMode::Append)?;

        assert_eq!(key, map.get_key(0), "Key is not correct");

        let num = map.num_elements(key)?;
        assert_eq!(1, num);

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
        let mut map = Map::default();
        let key = crate::key!(c"what");

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

        map.set(key, Value::Float(1e25), AppendMode::Replace)?;
        let res = map.get(key, 0)?;
        match res {
            Value::Float(val) => {
                assert_eq!(val, 1e25, "Value of `{key}` is not correct");
            }
            _ => panic!("Invalid type of `{key}`"),
        }
        let res = map.get_float_saturated(key, 0)?;
        assert_eq!(
            res, 9_999_999_562_023_526_247_432_192.0,
            "Value of `{key}` is not correct"
        );
        map.set(key, Value::Float(f64::MAX), AppendMode::Append)?;
        assert_eq!(&[1e25, f64::MAX], map.get_float_array(key)?);
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

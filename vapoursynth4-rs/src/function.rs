use crate::{api::Api, ffi, map::Map};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Function {
    handle: *const ffi::VSFunction,
    api: Api,
}

unsafe impl Send for Function {}

impl Function {
    pub(crate) unsafe fn from_ptr(ptr: *mut ffi::VSFunction, api: Api) -> Self {
        Self { handle: ptr, api }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *mut ffi::VSFunction {
        self.handle.cast_mut()
    }

    pub fn call(&mut self, in_: &Map, out: &mut Map) {
        unsafe {
            (self.api.callFunction)(self.as_ptr(), in_.as_ptr(), out.as_ptr());
        }
    }
}

impl Drop for Function {
    fn drop(&mut self) {
        unsafe { (self.api.freeFunction)(self.as_ptr()) }
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        unsafe { Self::from_ptr((self.api.addFunctionRef)(self.as_ptr()), self.api) }
    }
}

#[cfg(all(test, feature = "link-vs"))]
mod tests {
    use std::{
        ffi::c_void,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use super::*;
    use crate::{
        core::Core,
        key,
        map::{AppendMode, MapMut, Value},
    };

    unsafe extern "system-unwind" fn call(
        _: *const ffi::VSMap,
        output: *mut ffi::VSMap,
        data: *mut c_void,
        _: *mut ffi::VSCore,
        api: *const ffi::VSAPI,
    ) {
        unsafe {
            let data = &*data.cast::<Arc<AtomicUsize>>();
            data.fetch_add(1, Ordering::Relaxed);
            let mut output = MapMut::from_ptr(output, Api::from_ptr(api));
            let _ = output.set(key!(c"result"), Value::Int(42), AppendMode::Replace);
        }
    }

    unsafe extern "system-unwind" fn free(data: *mut c_void) {
        drop(unsafe { Box::from_raw(data.cast::<Arc<AtomicUsize>>()) });
    }

    #[test]
    fn map_holds_function_reference_until_removed() -> testresult::TestResult {
        let core = Core::builder().disable_auto_loading().build();
        let calls = Arc::new(AtomicUsize::new(0));
        let function = core.create_function(call, Box::new(calls.clone()), Some(free));
        let mut map = core.create_map();
        let key = key!(c"callback");
        map.consume_function(key, function.clone(), AppendMode::Replace)?;
        drop(function);

        let mut stored = map.get_function(key, 0)?;
        map.clear();
        assert_eq!(Arc::strong_count(&calls), 2);
        let mut output = core.create_map();
        stored.call(&map, &mut output);
        assert_eq!(output.get_int(key!(c"result"), 0)?, 42);
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        drop(stored);
        assert_eq!(Arc::strong_count(&calls), 1);
        Ok(())
    }
}

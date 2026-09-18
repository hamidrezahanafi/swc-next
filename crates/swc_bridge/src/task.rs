use napi::{Env, Result, Task};

use crate::NativeAbi;

/// Run bridge computation on the libuv pool and convert its output on the JS thread.
#[doc(hidden)]
pub struct BridgeTask<T> {
    compute: Option<Box<dyn FnOnce() -> Result<T> + Send>>,
}

impl<T> BridgeTask<T> {
    pub fn new(compute: impl FnOnce() -> Result<T> + Send + 'static) -> Self {
        Self {
            compute: Some(Box::new(compute)),
        }
    }
}

impl<T: NativeAbi + Send + 'static> Task for BridgeTask<T> {
    type Output = T;
    type JsValue = T::Abi;

    fn compute(&mut self) -> Result<T> {
        self.compute.take().expect("bridge task runs once")()
    }

    fn resolve(&mut self, _: Env, output: T) -> Result<Self::JsValue> {
        Ok(T::into_native(output))
    }
}

use super::ParamId;
use alloc::format;
use burn_common::stub::RwLock;
use core::ops::Deref;
use once_cell::sync::OnceCell;

/// Parameters are the fundamental building blocks of [modules](crate::module::Module) where they
/// serve as containers for [tensors](crate::tensor::Tensor) that can be updated during
/// training, and loaded during inference. If you don't want to save the tensors with a record
/// and/or don't want to update it during training, you don't need this type to wrap your tensor.
///
/// # Lazyness
///
/// The initialization of parameters can be lazy when created using
/// [uninitialized](Self::uninitialized), which can be done using an [initializer](crate::nn::Initializer).
///
/// This reduces the amount of allocations done when loading a model for inference without having
/// to create a custom initialization function only for inference.
///
/// ## Example
///
/// ```rust, ignore
/// let device = Device::default();
/// let config = ModuleConfig::default();
/// let record = Recorder::new().load("/path/to/module", &device);
///
/// // No tensor allocation
/// let module = config.init(device);
/// // Will use the tensor allocated for the record if the same device is used.
/// let module = module.load_record(record);
/// ```
pub struct Param<T: Parameter> {
    pub(crate) id: ParamId,
    state: OnceCell<T>,
    /// The locking is only required because of `lazy_device` and `lazy_is_require_grad`.
    ///
    /// Because of [once cell](OnceCell), we have a garanty that the initialization will only be called once,
    /// but it may be called at the same time as `lazy_device` and `lazy_is_require_grad`, which is
    /// when the lock is actually useful, waiting the the initialization to be completed before
    /// returning the value.
    initialization: RwLock<Option<Uninitialized<T>>>,
}

impl<T: Parameter> core::fmt::Display for Param<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("Param: {}", self.id).as_str())
    }
}

impl<T: Parameter> core::fmt::Debug for Param<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("Param: {}", self.id).as_str())
    }
}

/// Trait that defines that is necessary for a type to be a parameter.
pub trait Parameter: Clone + core::fmt::Debug + Send + Sync {
    /// The device type to be used.
    type Device: Clone;

    /// Fetch the device.
    fn device(&self) -> Self::Device;
    /// Fetch the gradient requirement.
    fn is_require_grad(&self) -> bool;
}

struct Uninitialized<P: Parameter> {
    init: Box<dyn Fn(&P::Device) -> P + Send + Sync>,
    device: P::Device,
    is_require_grad: bool,
}

impl<P: Parameter> Uninitialized<P> {
    fn initialize(&self) -> P {
        let init = &self.init;
        init(&self.device)
    }
}

impl<T: Parameter> Param<T> {
    /// Create a new parameter the is already initialized.
    pub fn initialized(id: ParamId, value: T) -> Self {
        Self {
            id,
            state: OnceCell::with_value(value),
            initialization: RwLock::new(None),
        }
    }

    /// Create a new parameter the is not initialized.
    pub fn uninitialized<F>(id: ParamId, init: F, device: T::Device, is_require_grad: bool) -> Self
    where
        F: Fn(&T::Device) -> T + Send + Sync + 'static,
    {
        Self {
            id,
            state: OnceCell::new(),
            initialization: RwLock::new(Some(Uninitialized {
                init: Box::new(init),
                device,
                is_require_grad,
            })),
        }
    }

    /// Gets the parameter value.
    ///
    /// # Returns
    ///
    /// The parameter value.
    pub fn val(&self) -> T {
        self.state
            .get_or_init(|| {
                let mut result = self.initialization.write().unwrap();
                let state = result.as_ref().expect("Should be something.");
                let tensor = state.initialize();

                *result = None;

                tensor
            })
            .clone()
    }

    /// Gets the parameter value while consuming the parameter.
    ///
    /// # Returns
    ///
    /// The parameter value.
    pub fn consume(self) -> (ParamId, T) {
        let state = self.state.into_inner();
        let tensor = match state {
            Some(tensor) => tensor,
            None => {
                let val = self.initialization.write();
                val.unwrap().as_ref().unwrap().initialize()
            }
        };

        (self.id, tensor)
    }

    /// Execute the given function on the inner value.
    pub fn map<F: Fn(T) -> T>(self, func: F) -> Self {
        let (id, tensor) = self.consume();
        let tensor = func(tensor);

        Self {
            id,
            state: OnceCell::with_value(tensor),
            initialization: RwLock::new(None),
        }
    }

    /// The device on which the parameter is or will be initialized.
    ///
    /// This should be used instead of [crate::tensor::Tensor::device], since using the tensor
    /// function requires a dereference, which triggers the initialization. This is only useful
    /// when the device is used for updating the tensor value, which has potentially not been
    /// initialized yet like loading a record.
    ///
    /// # Notes
    ///
    /// This is a crate private function, since users are not expected to use the device of an
    /// uninitialized module to then override its value. All low level functions should be provided
    /// by burn and should handle those details.
    pub(crate) fn lazy_device(&self) -> T::Device {
        let init = self.initialization.read().unwrap();

        match init.as_ref() {
            Some(value) => value.device.clone(),
            None => self.device(),
        }
    }

    /// The gradient requirement on which the parameter is or will be initialized.
    ///
    /// This should be used instead of [crate::tensor::Tensor::is_require_grad], since using the tensor
    /// function requires a dereference, which triggers the initialization. This is only useful
    /// when the boolean is used for updating the tensor value, which has potentially not been
    /// initialized yet like loading a record.
    ///
    /// # Notes
    ///
    /// This is a crate private function, since users are not expected to use `is_require_grad` of an
    /// uninitialized module to then override its value. All low level functions should be provided
    /// by burn and should handle those details.
    pub(crate) fn lazy_is_require_grad(&self) -> bool {
        let init = self.initialization.read().unwrap();

        match init.as_ref() {
            Some(value) => value.is_require_grad,
            None => self.is_require_grad(),
        }
    }
}

impl<T: Parameter> Clone for Param<T> {
    fn clone(&self) -> Self {
        Param::initialized(self.id.clone(), self.val())
    }
}

impl<T: Parameter> Deref for Param<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.state.get_or_init(|| {
            let mut result = self.initialization.write().unwrap();
            let state = result.as_ref().expect("Should be something.");
            let tensor = state.initialize();

            *result = None;

            tensor
        })
    }
}

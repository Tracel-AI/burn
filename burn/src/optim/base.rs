use crate::module::{LoadingError, Module, ParamId, State, StateNamed};
use crate::tensor::backend::{ADBackend, Backend};
use crate::tensor::{Gradients, Tensor};

pub trait Optimizer: Send + Sync {
    type Backend: ADBackend;

    fn update<const D: usize>(
        &mut self,
        id: &ParamId,
        tensor: &mut Tensor<Self::Backend, D>,
        grads: &Gradients,
    );

    /// Register the optimizer state for a given parameter.
    fn register_param_state<const D: usize>(
        &self,
        _id: &ParamId,
        _state: &mut StateNamed<<Self::Backend as Backend>::Elem>,
    ) {
        // By default there is no state to register
    }

    /// Load the optimizer state for a given parameter.
    fn load_param_state<const D: usize>(
        &mut self,
        _id: &ParamId,
        _state: &StateNamed<<Self::Backend as Backend>::Elem>,
        _device: &<Self::Backend as Backend>::Device,
    ) {
        // By default there is no state to load
    }

    /// Get the optimizer state for a given module.
    fn state<M: Module<Backend = Self::Backend>>(
        &self,
        module: &M,
    ) -> State<<Self::Backend as Backend>::Elem>
    where
        Self: Sized,
    {
        let mut state_named = StateNamed::new();
        module.register_optim_state(self, &mut state_named);
        State::StateNamed(state_named)
    }

    /// Load the optimizer state for a given module.
    fn load<M: Module<Backend = Self::Backend>>(
        &mut self,
        module: &M,
        state: &State<<Self::Backend as Backend>::Elem>,
    ) -> Result<(), LoadingError>
    where
        Self: Sized,
    {
        let state_named = match state {
            State::StateNamed(state) => state,
            _ => {
                return Err(LoadingError::new(
                    "Can't load state wrapper to fetch id and data".to_string(),
                ))
            }
        };

        module.load_optim_state(self, state_named);

        Ok(())
    }
}

use super::{load_with_id, state_with_id, Param};
use crate::module::{LoadingError, State, StateNamed};
use crate::optim::Optimizer;
use crate::tensor::{
    backend::{ADBackend, Backend},
    Data, Gradients, Tensor,
};

impl<const D: usize, B: Backend> Param<Tensor<B, D>> {
    pub fn num_params(&self) -> usize {
        self.value.shape().num_elements()
    }

    pub fn update_params<O: Optimizer<Backend = B>>(&mut self, grads: &Gradients, optim: &mut O)
    where
        B: ADBackend,
    {
        optim.update(&self.id, &mut self.value, grads);
    }

    pub fn load_optim_state<O: Optimizer<Backend = B>>(
        &self,
        optim: &mut O,
        state_optim: &StateNamed<B::Elem>,
    ) where
        B: ADBackend,
    {
        optim.load_param_state::<D>(&self.id, state_optim, &self.value.device());
    }

    pub fn register_optim_state<O: Optimizer<Backend = B>>(
        &self,
        optim: &O,
        state_optim: &mut StateNamed<B::Elem>,
    ) where
        B: ADBackend,
    {
        optim.register_param_state::<D>(&self.id, state_optim);
    }

    pub fn devices(&self) -> Vec<B::Device> {
        vec![self.value.device()]
    }

    pub fn to_device(&mut self, device: B::Device) {
        self.value = self.value.to_device(device);
    }

    pub fn state(&self) -> State<B::Elem> {
        let state = State::Data(self.value.to_data().serialize());

        state_with_id(self.id.clone(), state)
    }

    pub fn load(&mut self, state: &State<B::Elem>) -> Result<(), LoadingError> {
        let (id, state) = load_with_id(state)?;
        self.id = id.clone();

        match state {
            State::Data(data) => {
                self.value = Tensor::from_data_device(Data::from(data), self.value.device());
            }
            _ => return Err(LoadingError::new("Can't load tensor".to_string())),
        };

        Ok(())
    }

    pub fn inner(&self) -> Param<Tensor<B::InnerBackend, D>>
    where
        B: ADBackend,
    {
        Param::new(self.value.inner())
    }
}

impl<const D: usize, B: Backend> Param<Option<Tensor<B, D>>> {
    pub fn num_params(&self) -> usize {
        if let Some(value) = &self.value {
            return value.shape().num_elements();
        }

        0
    }

    pub fn update_params<O: Optimizer<Backend = B>>(&mut self, grads: &Gradients, optim: &mut O)
    where
        B: ADBackend,
    {
        if let Some(value) = &mut self.value {
            optim.update(&self.id, value, grads);
        }
    }

    pub fn load_optim_state<O: Optimizer<Backend = B>>(
        &self,
        optim: &mut O,
        state_optim: &StateNamed<B::Elem>,
    ) where
        B: ADBackend,
    {
        if let Some(value) = &self.value {
            optim.load_param_state::<D>(&self.id, state_optim, &value.device());
        }
    }

    pub fn register_optim_state<O: Optimizer<Backend = B>>(
        &self,
        optim: &O,
        state_optim: &mut StateNamed<B::Elem>,
    ) where
        B: ADBackend,
    {
        if let Some(_) = &self.value {
            optim.register_param_state::<D>(&self.id, state_optim);
        }
    }

    pub fn devices(&self) -> Vec<B::Device> {
        if let Some(value) = &self.value {
            return vec![value.device()];
        }

        vec![]
    }

    pub fn to_device(&mut self, device: B::Device) {
        if let Some(value) = &self.value {
            self.value = Some(value.to_device(device));
        }
    }

    pub fn state(&self) -> State<B::Elem> {
        let state = match &self.value {
            Some(value) => State::Data(value.to_data().serialize()),
            None => State::StateNamed(StateNamed::new()),
        };

        state_with_id(self.id.clone(), state)
    }

    pub fn load(&mut self, state: &State<B::Elem>) -> Result<(), LoadingError> {
        let (id, state) = load_with_id(state)?;
        self.id = id.clone();

        let data = match state {
            State::Data(data) => data,
            _ => {
                return Err(LoadingError::new(
                    "Can't load Option<Tensor> from NamedState".to_string(),
                ))
            }
        };

        if let Some(value) = &self.value {
            self.value = Some(Tensor::from_data_device(Data::from(data), value.device()));
        }

        Ok(())
    }

    pub fn inner(&self) -> Param<Option<Tensor<B::InnerBackend, D>>>
    where
        B: ADBackend,
    {
        match &self.value {
            Some(tensor) => Param::new(Some(tensor.inner())),
            None => Param::new(None),
        }
    }
}

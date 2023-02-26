use burn_tensor::{backend::Backend, container::TensorContainer, Tensor};

use crate::{
    graph::NodeRef,
    tensor::{ADTensor, BackwardTensor},
};

pub type GradID = String;
#[derive(Default)]
pub struct Gradients<B: Backend> {
    container: TensorContainer<B, GradID>,
}

impl<B: Backend> Gradients<B> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn consume<const D: usize>(
        &mut self,
        tensor: &BackwardTensor<B, D>,
    ) -> B::TensorPrimitive<D> {
        self.container
            .get(&tensor.node.id.value)
            .map(|tensor| tensor.into_primitive())
            .unwrap_or_else(|| B::zeros(B::shape(&tensor.primitive), &B::device(&tensor.primitive)))
    }
    pub fn get<const D: usize>(&self, tensor: &ADTensor<B, D>) -> Option<B::TensorPrimitive<D>> {
        self.container
            .get(&tensor.node.id.value)
            .map(|tensor| tensor.into_primitive())
    }

    pub fn update<const D: usize>(&mut self, node: NodeRef, value: B::TensorPrimitive<D>) {
        if let Some(tensor_old) = self.container.remove(&node.id.value) {
            self.container.register(
                node.id.value.clone(),
                Tensor::from_primitive(value).add(tensor_old),
            );
        } else {
            self.container
                .register(node.id.value.clone(), Tensor::from_primitive(value));
        }
    }
}

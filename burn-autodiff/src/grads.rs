use burn_tensor::{backend::Backend, container::TensorContainer, Tensor};

use crate::{
    graph::{NodeRef, Requirement},
    tensor::ADTensor,
};

pub type GradID = String;

/// Gradients container used during the backward pass.
pub struct Gradients {
    container: TensorContainer<GradID>,
}

type TensorPrimitive<B, const D: usize> = <B as Backend>::TensorPrimitive<D>;

impl Gradients {
    pub fn new<B: Backend, const D: usize>(
        root_node: NodeRef,
        root_tensor: TensorPrimitive<B, D>,
    ) -> Self {
        let mut gradients = Self {
            container: TensorContainer::new(),
        };
        gradients.register::<B, D>(
            root_node,
            B::ones(B::shape(&root_tensor), &B::device(&root_tensor)),
        );
        gradients
    }
    /// Consume the gradient for a given tensor.
    ///
    /// Each tensor should be consumed exactly 1 time if its gradient is only required during the
    /// backward pass.
    pub fn consume<B: Backend, const D: usize>(&mut self, node: &NodeRef) -> TensorPrimitive<B, D> {
        match node.requirement {
            Requirement::Grad => self
                .container
                .get::<B, D>(&node.id.value)
                .map(|tensor| tensor.into_primitive())
                .expect("Can't consume the gradients before they are registered at least once."),
            Requirement::GradInBackward => self
                .container
                .remove::<B, D>(&node.id.value)
                .map(|tensor| tensor.into_primitive())
                .expect("Can't consume the gradients before they are registered at least once."),
            Requirement::None => panic!("Trying to consume the gradients for an untracked tensor"),
        }
    }

    pub fn get<B: Backend, const D: usize>(
        &self,
        tensor: &ADTensor<B, D>,
    ) -> Option<TensorPrimitive<B, D>> {
        self.container
            .get::<B, D>(&tensor.node.id.value)
            .map(|tensor| tensor.into_primitive())
    }

    pub fn register<B: Backend, const D: usize>(
        &mut self,
        node: NodeRef,
        value: TensorPrimitive<B, D>,
    ) {
        if let Some(tensor_old) = self.container.remove::<B, D>(&node.id.value) {
            self.container.register(
                node.id.value.clone(),
                Tensor::from_primitive(value).add(tensor_old),
            );
        } else {
            self.container
                .register::<B, D>(node.id.value.clone(), Tensor::from_primitive(value));
        }
    }
}

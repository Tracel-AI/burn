use super::ADKind;
use crate::{
    execute_ops,
    graph::node::ForwardNodeRef,
    tensor::{
        ops::{TensorOpsAny, TensorOpsUtilities},
        Backend, Data, Element, Shape, Tensor, TensorTrait, TensorType,
    },
};

#[derive(Debug, Clone)]
pub struct ADTensor<P, const D: usize, T> {
    pub node: ForwardNodeRef<T>,
    pub shape: Shape<D>,
    pub kind: ADKind<P>,
}

impl<P: 'static, const D: usize, T: 'static> TensorOpsAny<P, D> for ADTensor<P, D, T> {
    fn to_any(self) -> Box<dyn std::any::Any> {
        Box::new(self)
    }

    fn from_any(any: Box<dyn std::any::Any>) -> Self {
        let me: Box<ADTensor<P, D, T>> = any.downcast().unwrap();
        *me
    }
}

impl<T, P, const D: usize> TensorOpsUtilities<P, D> for ADTensor<P, D, T>
where
    P: Element,
    T: TensorTrait<P, D>,
{
    fn shape(&self) -> &Shape<D> {
        &self.shape
    }

    fn into_data(self) -> Data<P, D> {
        self.tensor().into_data()
    }
    fn to_data(&self) -> Data<P, D> {
        self.tensor().to_data()
    }
}

impl<T, P, const D: usize> ADTensor<P, D, T>
where
    P: Element,
    T: TensorTrait<P, D>,
{
    pub fn from_tensor(tensor: T) -> Self {
        let node = execute_ops!(
            init tensor.clone()
        );

        let shape = tensor.shape().clone();
        let kind = ADKind::new();
        Self { node, shape, kind }
    }

    pub fn from_existing(&self, node: ForwardNodeRef<T>) -> Self {
        let shape = self.shape.clone();
        let kind = self.kind.clone();

        Self { node, shape, kind }
    }
}

impl<T: Clone + std::fmt::Debug, P, const D: usize> ADTensor<P, D, T> {
    pub fn tensor(&self) -> T {
        self.node.state.value()
    }
}

#[cfg(test)]
pub mod helper {
    use super::*;

    #[cfg(feature = "ndarray")]
    mod helper_impl {
        use super::*;
        use crate::tensor::{backend::ndarray::NdArrayTensor, Data};

        pub type TestADTensor<P, const D: usize> = ADTensor<P, D, NdArrayTensor<P, D>>;

        impl<P: Element + ndarray::ScalarOperand + ndarray::LinalgScalar, const D: usize>
            TestADTensor<P, D>
        {
            pub fn from_data(data: Data<P, D>) -> Self {
                let tensor = NdArrayTensor::from_data(data);
                ADTensor::from_tensor(tensor)
            }
        }
    }
    pub use helper_impl::*;

    #[cfg(feature = "tch")]
    #[cfg(not(feature = "ndarray"))]
    mod helper_impl {
        use super::*;
        use crate::tensor::backend::tch::TchTensor;

        pub type TestADTensor<P, const D: usize> = ADTensor<P, D, TchTensor<P, D>>;
        impl<P: Element + tch::kind::Element + Into<f64>, const D: usize> TestADTensor<P, D> {
            pub fn from_data(data: Data<P, D>) -> Self {
                let tensor = TchTensor::from_data(data, tch::Device::Cpu);
                ADTensor::from_tensor(tensor)
            }
        }
    }
    pub use helper_impl::*;
}

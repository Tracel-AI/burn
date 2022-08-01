use crate::tensor::backend::Backend;
use crate::tensor::ops::*;
use crate::tensor::{Data, Distribution, Shape};

#[derive(Debug, Clone)]
pub struct Tensor<const D: usize, B: Backend> {
    pub(crate) value: B::TensorPrimitive<D>,
}

impl<const D: usize, B> Tensor<D, B>
where
    B: Backend,
{
    pub fn new(tensor: B::TensorPrimitive<D>) -> Self {
        Self { value: tensor }
    }

    pub fn reshape<const D2: usize>(&self, shape: Shape<D2>) -> Tensor<D2, B> {
        Tensor::new(self.value.reshape(shape))
    }

    pub fn shape(&self) -> &Shape<D> {
        self.value.shape()
    }

    pub fn into_data(self) -> Data<B::Elem, D> {
        self.value.into_data()
    }

    pub fn to_data(&self) -> Data<B::Elem, D> {
        self.value.to_data()
    }

    pub fn zeros_like<const D2: usize>(other: &Tensor<D2, B>) -> Tensor<D2, B> {
        todo!()
        // Tensor::new(B::zeros(other.shape()), other.value.get_device())
    }

    pub fn new_like_empty(&self) -> Self {
        Self::new(self.value.new_like_empty())
    }

    pub fn new_like_random(&self, distribution: Distribution<B::Elem>) -> Self {
        Self::new(self.value.new_like_random(distribution))
    }

    pub fn new_like_data(&self, data: Data<B::Elem, D>) -> Self {
        Self::new(self.value.new_like_data(data))
    }

    pub fn new_like_zeros(&self) -> Self {
        Self::new(self.value.new_like_zeros())
    }

    pub fn new_like_ones(&self) -> Self {
        Self::new(self.value.new_like_ones())
    }

    pub fn new_fork_empty<const D2: usize>(&self, shape: Shape<D2>) -> Tensor<D2, B> {
        Tensor::new(self.value.new_fork_empty(shape))
    }

    pub fn new_fork_random<const D2: usize>(
        &self,
        shape: Shape<D2>,
        distribution: Distribution<B::Elem>,
    ) -> Tensor<D2, B> {
        Tensor::new(self.value.new_fork_random(shape, distribution))
    }

    pub fn new_fork_data<const D2: usize>(&self, data: Data<B::Elem, D2>) -> Tensor<D2, B> {
        Tensor::new(self.value.new_fork_data(data))
    }

    pub fn new_fork_zeros<const D2: usize>(&self, shape: Shape<D2>) -> Tensor<D2, B> {
        Tensor::new(self.value.new_fork_zeros(shape))
    }

    pub fn new_fork_ones<const D2: usize>(&self, shape: Shape<D2>) -> Tensor<D2, B> {
        Tensor::new(self.value.new_fork_ones(shape))
    }

    pub fn add(&self, other: &Self) -> Self {
        Self::new(self.value.add(&other.value))
    }

    pub fn add_scalar(&self, other: &B::Elem) -> Self {
        Self::new(self.value.add_scalar(&other))
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self::new(self.value.sub(&other.value))
    }

    pub fn sub_scalar(&self, other: &B::Elem) -> Self {
        Self::new(self.value.sub_scalar(&other))
    }

    pub fn transpose(&self) -> Self {
        Self::new(self.value.transpose())
    }

    pub fn matmul(&self, other: &Self) -> Self {
        Self::new(self.value.matmul(&other.value))
    }

    pub fn neg(&self) -> Self {
        Self::new(self.value.neg())
    }

    pub fn mul(&self, other: &Self) -> Self {
        Self::new(self.value.mul(&other.value))
    }

    pub fn mul_scalar(&self, other: &B::Elem) -> Self {
        Self::new(self.value.mul_scalar(&other))
    }

    pub fn random(shape: Shape<D>, distribution: Distribution<B::Elem>) -> Self {
        let tensor = B::random(shape, distribution, B::Device::default());
        Self::new(tensor)
    }

    pub fn zeros(shape: Shape<D>) -> Self {
        let tensor = B::zeros(shape, B::Device::default());
        Self::new(tensor)
    }

    pub fn ones(shape: Shape<D>) -> Self {
        let tensor = B::ones(shape, B::Device::default());
        Self::new(tensor)
    }

    pub fn from_data(data: Data<B::Elem, D>) -> Self {
        let tensor = B::from_data(data, B::Device::default());
        Tensor::new(tensor)
    }

    pub fn from_data_device(data: Data<B::Elem, D>, device: B::Device) -> Self {
        let tensor = B::from_data(data, device);
        Tensor::new(tensor)
    }

    pub fn index<const D2: usize>(&self, indexes: [std::ops::Range<usize>; D2]) -> Self {
        Self::new(self.value.index(indexes))
    }

    pub fn update(&mut self, other: Self) {
        self.value = other.value;
    }

    pub fn index_assign<const D2: usize>(
        &self,
        indexes: [std::ops::Range<usize>; D2],
        values: &Self,
    ) -> Self {
        Self::new(self.value.index_assign(indexes, &values.value))
    }

    pub fn unsqueeze<const D2: usize>(&self) -> Tensor<D2, B> {
        if D2 < D {
            panic!(
                "Can't unsqueeze smaller tensor, got dim {}, expected > {}",
                D2, D
            )
        }

        let mut dims = [1; D2];
        let num_ones = D2 - D;
        let shape = self.shape();

        for i in 0..D {
            dims[i + num_ones] = shape.dims[i];
        }

        let shape = Shape::new(dims);
        self.reshape(shape)
    }
}

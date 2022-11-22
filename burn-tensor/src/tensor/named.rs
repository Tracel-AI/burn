use crate::backend::Backend;
use crate::{Shape, Tensor};

pub trait Dim {}
pub trait NamedDims<B: Backend> {
    type Tensor;
}

// impl<B: Backend, D1> NamedDims<B> for (D1,)
// where
//     B: Backend,
//     D1: Dim,
// {
//     type Tensor = Tensor<B, 1>;
// }
//
// impl<B: Backend, D1, D2> NamedDims<B> for (D1, D2)
// where
//     B: Backend,
//     D1: Dim,
//     D2: Dim,
// {
//     type Tensor = Tensor<B, 2>;
// }

impl<B: Backend, D1, D2, D3> NamedDims<B> for (D1, D2, D3)
where
    B: Backend,
    D1: Dim,
    D2: Dim,
    D3: Dim,
{
    type Tensor = Tensor<B, 3>;
}

// impl<B: Backend, D1, D2, D3, D4> NamedDims<B> for (D1, D2, D3, D4)
// where
//     B: Backend,
//     D1: Dim,
//     D2: Dim,
//     D3: Dim,
//     D4: Dim,
// {
//     type Tensor = Tensor<B, 4>;
// }
//
// impl<B: Backend, D1, D2, D3, D4, D5> NamedDims<B> for (D1, D2, D3, D4, D5)
// where
//     B: Backend,
//     D1: Dim,
//     D2: Dim,
//     D3: Dim,
//     D4: Dim,
//     D5: Dim,
// {
//     type Tensor = Tensor<B, 5>;
// }
//
// impl<B: Backend, D1, D2, D3, D4, D5, D6> NamedDims<B> for (D1, D2, D3, D4, D5, D6)
// where
//     B: Backend,
//     D1: Dim,
//     D2: Dim,
//     D3: Dim,
//     D4: Dim,
//     D5: Dim,
//     D6: Dim,
// {
//     type Tensor = Tensor<B, 6>;
// }

pub struct NamedTensor<B: Backend, D: NamedDims<B>> {
    tensor: D::Tensor,
}

macro_rules! dim {
    ($name:ident) => {
        pub struct $name;
        impl Dim for $name {}
    };
}

pub trait Permut<N, const D1: usize, const D2: usize> {
    fn permut(&self) -> N;
}

impl<B: Backend, const D: usize, ND> NamedTensor<B, ND>
where
    ND: NamedDims<B, Tensor = Tensor<B, D>>,
{
    pub fn from_tensor(tensor: ND::Tensor) -> Self {
        Self { tensor }
    }

    pub fn mul(&self, rhs: &Self) -> Self {
        Self::from_tensor(self.tensor.mul(&rhs.tensor))
    }

    pub fn reshape<const D2: usize, S, ND2>(&self, shape: S, _: ND2) -> NamedTensor<B, ND2>
    where
        S: Into<Shape<D2>>,
        ND2: NamedDims<B, Tensor = Tensor<B, D2>>,
    {
        NamedTensor::from_tensor(self.tensor.reshape(shape))
    }

    pub fn permut<ND2, const D1: usize, const D2: usize>(&self) -> NamedTensor<B, ND2>
    where
        ND2: NamedDims<B, Tensor = Tensor<B, D>>,
        Self: Permut<NamedTensor<B, ND2>, D1, D2>,
    {
        Permut::permut(self)
    }
}

macro_rules! generate_permut {
    (dim3: $output:ty, $dim1:expr, $dim2:expr) => {
        impl<B: Backend, D1: Dim, D2: Dim, D3: Dim> Permut<NamedTensor<B, $output>, $dim1, $dim2>
            for NamedTensor<B, (D1, D2, D3)>
        {
            fn permut(&self) -> NamedTensor<B, $output> {
                NamedTensor::from_tensor(self.tensor.swap_dims($dim1, $dim2))
            }
        }
    };
}

generate_permut!(dim3: (D2, D1, D3), 0, 1);
generate_permut!(dim3: (D3, D2, D1), 0, 2);
generate_permut!(dim3: (D1, D3, D2), 1, 2);

impl<B: Backend, X: Dim, Y: Dim> NamedTensor<B, (Batch, X, Y)> {
    pub fn matmul<Z: Dim>(
        &self,
        rhs: NamedTensor<B, (Batch, Y, Z)>,
    ) -> NamedTensor<B, (Batch, X, Z)> {
        let tensor = self.tensor.matmul(&rhs.tensor);
        NamedTensor { tensor }
    }
}

dim!(Batch);
dim!(DModel);
dim!(NFeatures);

fn allo<B: Backend>(
    input: NamedTensor<B, (Batch, DModel, NFeatures)>,
    weights: NamedTensor<B, (Batch, DModel, DModel)>,
) -> NamedTensor<B, (Batch, NFeatures, DModel)> {
    // weights.matmul(input);
    let input = input.permut::<_, 1, 2>();
    input.matmul(weights)
    // input.matmul(weights)
}

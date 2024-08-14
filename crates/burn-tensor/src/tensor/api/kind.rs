use crate::backend::Backend;
use crate::{Dense, Sparse, SparseRepr, TensorRepr};
use core::marker::PhantomData;

/// A type-level representation of the kind of a float tensor
#[derive(Clone, Debug)]
pub struct Float;

/// A type-level representation of the kind of a int tensor.
#[derive(Clone, Debug)]
pub struct Int;

/// A type-level representation of the kind of a bool tensor.
#[derive(Clone, Debug)]
pub struct Bool;

#[derive(Debug, Clone)]
/// A primitive tensor representation.
pub enum TensorPrimitive<B: Backend, const D: usize> {
    /// Float tensor primitive.
    Float(B::FloatTensorPrimitive<D>),
    /// Quantized float tensor primitive.
    QFloat(B::QuantizedTensorPrimitive<D>),
}

impl<B: Backend, const D: usize> TensorPrimitive<B, D> {
    /// Returns the full tensor representation.
    pub fn tensor(self) -> B::FloatTensorPrimitive<D> {
        match self {
            Self::QFloat(tensor) => B::dequantize(tensor),
            Self::Float(tensor) => tensor,
        }
    }
}

/// A type-level representation of the kind of a tensor.
pub trait TensorKind<B: Backend, R: TensorRepr<B> = Dense>: Clone + core::fmt::Debug {
    /// The primitive type of the tensor.
    type Primitive<const D: usize>: Clone + core::fmt::Debug + Send;

    /// The primitive type of the tensor when dense.
    type DensePrimitive<const D: usize>: Clone + core::fmt::Debug + Send;

    /// The name of the tensor kind.
    fn name() -> &'static str;

    /// The representation of the tensor kind.
    fn representation() -> &'static str {
        R::name()
    }
}

// impl<B: Backend> TensorKind<B> for Float {
//     type Primitive<const D: usize> = TensorPrimitive<B, D>;
//     fn name() -> &'static str {
//         "Float"
//     }
// }

// impl<B: Backend, R: SparseRepr<B>> TensorKind<B, Sparse<R, B>> for Float {
//     type Primitive<const D: usize> = R::FloatTensorPrimitive<D>;
//     fn name() -> &'static str {
//         <Self as TensorKind<B, Dense>>::name()
//     }
// }

// impl<B: Backend> TensorKind<B> for Int {
//     type Primitive<const D: usize> = B::IntTensorPrimitive<D>;
//     fn name() -> &'static str {
//         "Int"
//     }
// }

// impl<B: Backend, R: SparseRepr<B>> TensorKind<B, Sparse<R, B>> for Int {
//     type Primitive<const D: usize> = R::IntTensorPrimitive<D>;

//     fn name() -> &'static str {
//         <Self as TensorKind<B, Dense>>::name()
//     }
// }

// impl<B: Backend> TensorKind<B> for Bool {
//     type Primitive<const D: usize> = B::BoolTensorPrimitive<D>;
//     fn name() -> &'static str {
//         "Bool"
//     }
// }

// impl<B: Backend, R: SparseRepr<B>> TensorKind<B, Sparse<R, B>> for Bool {
//     type Primitive<const D: usize> = R::BoolTensorPrimitive<D>;

//     fn name() -> &'static str {
//         <Self as TensorKind<B, Dense>>::name()
//     }
// }

impl<B: Backend, R: TensorRepr<B>> TensorKind<B, R> for Bool {
    type DensePrimitive<const D: usize> = B::BoolTensorPrimitive<D>;
    type Primitive<const D: usize> = R::Primitive<Bool, D>;

    fn name() -> &'static str {
        "Bool"
    }
}

impl<B: Backend, R: TensorRepr<B>> TensorKind<B, R> for Float {
    type DensePrimitive<const D: usize> = TensorPrimitive<B, D>;
    type Primitive<const D: usize> = R::Primitive<Float, D>;

    fn name() -> &'static str {
        "Float"
    }
}

impl<B: Backend, R: TensorRepr<B>> TensorKind<B, R> for Int {
    type DensePrimitive<const D: usize> = B::IntTensorPrimitive<D>;
    type Primitive<const D: usize> = R::Primitive<Int, D>;

    fn name() -> &'static str {
        "Int"
    }
}

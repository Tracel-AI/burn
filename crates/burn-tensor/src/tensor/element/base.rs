use core::cmp::Ordering;

use crate::{cast::ToElement, Distribution};
use half::{bf16, f16};
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// Element trait for tensor.
pub trait Element:
    ToElement
    + ElementRandom
    + ElementConversion
    + ElementPrecision
    + ElementComparison
    + bytemuck::Pod
    + core::fmt::Debug
    + core::fmt::Display
    + Default
    + Send
    + Sync
    + Copy
    + 'static
{
    /// The dtype of the element.
    fn dtype() -> DType;
}

/// Element conversion trait for tensor.
pub trait ElementConversion {
    /// Converts an element to another element.
    ///
    /// # Arguments
    ///
    /// * `elem` - The element to convert.
    ///
    /// # Returns
    ///
    /// The converted element.
    fn from_elem<E: ToElement>(elem: E) -> Self;

    /// Converts and returns the converted element.
    fn elem<E: Element>(self) -> E;
}

/// Element trait for random value of a tensor.
pub trait ElementRandom {
    /// Returns a random value for the given distribution.
    ///
    /// # Arguments
    ///
    /// * `distribution` - The distribution to sample from.
    /// * `rng` - The random number generator.
    ///
    /// # Returns
    ///
    /// The random value.
    fn random<R: RngCore>(distribution: Distribution, rng: &mut R) -> Self;
}

/// Element ordering trait.
pub trait ElementComparison {
    /// Returns and [Ordering] between `self` and `other`.
    fn cmp(&self, other: &Self) -> Ordering;
}

/// Element precision trait for tensor.
#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Precision {
    /// Double precision, e.g. f64.
    Double,

    /// Full precision, e.g. f32.
    Full,

    /// Half precision, e.g. f16.
    Half,

    /// Other precision.
    Other,
}

/// Element precision trait for tensor.
pub trait ElementPrecision {
    /// Returns the precision of the element.
    fn precision() -> Precision;
}

/// Macro to implement the element trait for a type.
#[macro_export]
macro_rules! make_element {
    (
        ty $type:ident $precision:expr,
        convert $convert:expr,
        random $random:expr,
        cmp $cmp:expr,
        dtype $dtype:expr

    ) => {
        impl Element for $type {
            fn dtype() -> $crate::DType {
                $dtype
            }
        }

        impl ElementConversion for $type {
            fn from_elem<E: ToElement>(elem: E) -> Self {
                #[allow(clippy::redundant_closure_call)]
                $convert(&elem)
            }
            fn elem<E: Element>(self) -> E {
                E::from_elem(self)
            }
        }

        impl ElementPrecision for $type {
            fn precision() -> Precision {
                $precision
            }
        }

        impl ElementRandom for $type {
            fn random<R: RngCore>(distribution: Distribution, rng: &mut R) -> Self {
                #[allow(clippy::redundant_closure_call)]
                $random(distribution, rng)
            }
        }

        impl ElementComparison for $type {
            fn cmp(&self, other: &Self) -> Ordering {
                let a = self.elem::<$type>();
                let b = other.elem::<$type>();
                #[allow(clippy::redundant_closure_call)]
                $cmp(&a, &b)
            }
        }
    };
}

make_element!(
    ty f64 Precision::Double,
    convert |elem: &dyn ToElement| elem.to_f64(),
    random |distribution: Distribution, rng: &mut R| distribution.sampler(rng).sample(),
    cmp |a: &f64, b: &f64| a.total_cmp(b),
    dtype DType::F64
);

make_element!(
    ty f32 Precision::Full,
    convert |elem: &dyn ToElement| elem.to_f32(),
    random |distribution: Distribution, rng: &mut R| distribution.sampler(rng).sample(),
    cmp |a: &f32, b: &f32| a.total_cmp(b),
    dtype DType::F32
);

make_element!(
    ty i64 Precision::Double,
    convert |elem: &dyn ToElement| elem.to_i64(),
    random |distribution: Distribution, rng: &mut R| distribution.sampler(rng).sample(),
    cmp |a: &i64, b: &i64| Ord::cmp(a, b),
    dtype DType::I64
);

make_element!(
    ty u64 Precision::Double,
    convert |elem: &dyn ToPrimitive| elem.to_u64().unwrap(),
    random |distribution: Distribution, rng: &mut R| distribution.sampler(rng).sample(),
    cmp |a: &u64, b: &u64| Ord::cmp(a, b),
    dtype DType::U64
);

make_element!(
    ty i32 Precision::Full,
    convert |elem: &dyn ToElement| elem.to_i32(),
    random |distribution: Distribution, rng: &mut R| distribution.sampler(rng).sample(),
    cmp |a: &i32, b: &i32| Ord::cmp(a, b),
    dtype DType::I32
);

make_element!(
    ty u32 Precision::Full,
    convert |elem: &dyn ToElement| elem.to_u32(),
    random |distribution: Distribution, rng: &mut R| distribution.sampler(rng).sample(),
    cmp |a: &u32, b: &u32| Ord::cmp(a, b),
    dtype DType::U32
);

make_element!(
    ty i16 Precision::Half,
    convert |elem: &dyn ToElement| elem.to_i16(),
    random |distribution: Distribution, rng: &mut R| distribution.sampler(rng).sample(),
    cmp |a: &i16, b: &i16| Ord::cmp(a, b),
    dtype DType::I16
);

make_element!(
    ty i8 Precision::Other,
    convert |elem: &dyn ToElement| elem.to_i8(),
    random |distribution: Distribution, rng: &mut R| distribution.sampler(rng).sample(),
    cmp |a: &i8, b: &i8| Ord::cmp(a, b),
    dtype DType::I8
);

make_element!(
    ty u8 Precision::Other,
    convert |elem: &dyn ToElement| elem.to_u8(),
    random |distribution: Distribution, rng: &mut R| distribution.sampler(rng).sample(),
    cmp |a: &u8, b: &u8| Ord::cmp(a, b),
    dtype DType::U8
);

make_element!(
    ty f16 Precision::Half,
    convert |elem: &dyn ToElement| f16::from_f32(elem.to_f32()),
    random |distribution: Distribution, rng: &mut R| {
        let sample: f32 = distribution.sampler(rng).sample();
        f16::from_elem(sample)
    },
    cmp |a: &f16, b: &f16| a.total_cmp(b),
    dtype DType::F16
);
make_element!(
    ty bf16 Precision::Half,
    convert |elem: &dyn ToElement| bf16::from_f32(elem.to_f32()),
    random |distribution: Distribution, rng: &mut R| {
        let sample: f32 = distribution.sampler(rng).sample();
        bf16::from_elem(sample)
    },
    cmp |a: &bf16, b: &bf16| a.total_cmp(b),
    dtype DType::BF16
);

make_element!(
    ty bool Precision::Other,
    convert |elem: &dyn ToElement| elem.to_u8() != 0,
    random |distribution: Distribution, rng: &mut R| {
        let sample: u8 = distribution.sampler(rng).sample();
        bool::from_elem(sample)
    },
    cmp |a: &bool, b: &bool| Ord::cmp(a, b),
    dtype DType::Bool
);

#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum DType {
    F64,
    F32,
    F16,
    BF16,
    I64,
    I32,
    I16,
    I8,
    U64,
    U32,
    U8,
    Bool,
}

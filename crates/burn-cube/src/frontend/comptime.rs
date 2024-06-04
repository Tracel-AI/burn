use crate::{
    frontend::{CubeContext, CubeType},
    ir::Variable,
    runtime, unexpanded,
};

use super::{CubeElem, ExpandElement, UInt, Vectorized};

#[derive(Clone, Copy)]
/// Encapsulates a value to signify it must be used at compilation time rather than in the kernel
///
/// Use `Comptime<Option<T>>` to have an alternate runtime behaviour if the compilation time value is not present
pub struct Comptime<T> {
    pub(crate) inner: T,
}

impl<T> Comptime<T> {
    /// Create a new Comptime. Useful when hardcoding values in
    /// Cube kernels. For instance:
    /// if Comptime::new(false) {...} never generates the inner code block
    pub fn new(_inner: T) -> Self {
        unexpanded!()
    }

    /// Get the inner value of a Comptime. For instance:
    /// let c = Comptime::new(false);
    /// if Comptime::get(c) {...}
    pub fn get(_comptime: Self) -> T {
        unexpanded!()
    }

    pub fn map<R, F: Fn(T) -> R>(_comptime: Self, _closure: F) -> Comptime<R> {
        unexpanded!()
    }

    pub fn map_expand<R, F: Fn(&mut CubeContext, T) -> R>(
        context: &mut CubeContext,
        inner: T,
        closure: F,
    ) -> R {
        closure(context, inner)
    }
}

impl<T: CubeType + Into<T::ExpandType>> Comptime<Option<T>> {
    /// Map a Comptime optional to a Comptime boolean that tell
    /// whether the optional contained a value
    pub fn is_some(comptime: Self) -> Comptime<bool> {
        Comptime::new(comptime.inner.is_some())
    }

    /// Return the inner value of the Comptime if it exists,
    /// otherwise tell how to compute it at runtime
    pub fn unwrap_or_else<F>(_comptime: Self, mut _alt: F) -> T
    where
        F: FnOnce() -> T,
    {
        unexpanded!()
    }

    /// Expanded version of unwrap_or_else
    pub fn unwrap_or_else_expand<F>(
        context: &mut CubeContext,
        t: Option<T>,
        mut alt: F,
    ) -> <T as CubeType>::ExpandType
    where
        F: FnOnce(&mut CubeContext) -> T::ExpandType,
    {
        match t {
            Some(t) => t.into(),
            None => alt(context),
        }
    }
}

impl<T: Clone> CubeType for Comptime<T> {
    type ExpandType = T;
}

impl<T: Vectorized> Comptime<T> {
    pub fn vectorization(_state: T) -> Comptime<UInt> {
        unexpanded!()
    }

    pub fn vectorization_expand(_context: &mut CubeContext, state: T) -> Comptime<UInt> {
        Comptime::new(UInt::new(state.vectorization_factor() as u32))
    }
}

impl<T: Into<ExpandElement>> Comptime<T> {
    pub fn runtime(_comptime: Self) -> T {
        unexpanded!()
    }
    pub fn runtime_expand(_context: &mut CubeContext, comptime: Self) -> ExpandElement {
        comptime.inner.into()
    }
}

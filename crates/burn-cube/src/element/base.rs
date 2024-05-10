use alloc::rc::Rc;
use burn_jit::gpu::{Item, Variable};

/// Types used in a cube function must implement this trait
///
/// Variables whose values will be known at runtime must
/// have ExpandElement as associated type (using RuntimeType)
/// Variables whose values will be known at compile time
/// must have the primitive type as associated type
///
/// Note: Cube functions should be written using CubeTypes,
/// so that the code generated uses the associated ExpandType.
/// This allows Cube code to not necessitate cloning, which is cumbersome
/// in algorithmic code. The necessary cloning will automatically appear in
/// the generated code.
pub trait CubeType {
    type ExpandType: Clone;
}

#[derive(new, Clone, Debug)]
/// Reference to a JIT variable
/// It's the expand element that is actually kept in the variable pool
pub struct ExpandElement {
    pub(crate) inner: Rc<Variable>,
}

impl ExpandElement {
    /// Returns the Item of the variable
    pub fn item(&self) -> Item {
        self.inner.item()
    }
}

impl core::ops::Deref for ExpandElement {
    type Target = Variable;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl From<ExpandElement> for Variable {
    fn from(value: ExpandElement) -> Self {
        (*value.inner).clone()
    }
}

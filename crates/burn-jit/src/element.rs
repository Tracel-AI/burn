use burn_cube::CubeElement;

/// The base element trait for the jit backend.
pub trait JitElement: burn_tensor::Element + CubeElement {}

/// The float element type for the jit backend.
pub trait FloatElement: JitElement {}

/// The int element type for the jit backend.
pub trait IntElement: JitElement {}

impl JitElement for u32 {}

impl JitElement for i32 {}

impl JitElement for f32 {}

impl JitElement for half::f16 {}

impl JitElement for half::bf16 {}
impl FloatElement for f32 {}
impl FloatElement for half::bf16 {}
impl FloatElement for half::f16 {}
impl IntElement for i32 {}

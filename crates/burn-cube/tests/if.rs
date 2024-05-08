use burn_cube::{cube, if_expand, CubeContext, Float, FloatKind_, F32_};
use burn_jit::gpu;
use burn_jit::gpu::FloatKind::F32;
use burn_jit::gpu::{Elem, Item, Variable};

#[cube]
pub fn if_greater<F: FloatKind_>(lhs: Float<F>) {
    if lhs > float_new::<F>(0.0) {
        let _ = lhs + float_new::<F>(4.0);
    }
}

#[test]
fn cube_if_test() {
    let mut context = CubeContext::root();

    let lhs = context.create_local(Item::Scalar(Elem::Float(F32)));

    if_greater::expand::<F32_>(&mut context, lhs);
    let scope = context.into_scope();

    assert_eq!(format!("{:?}", scope.operations), gpu_macro_ref());
}

fn gpu_macro_ref() -> String {
    let mut context = CubeContext::root();
    let item = Item::Scalar(Elem::Float(F32));
    let lhs = context.create_local(item);

    let mut scope = context.into_scope();
    let cond = scope.create_local(Item::Scalar(Elem::Bool));
    let lhs: Variable = lhs.into();
    let y = scope.create_local(item);

    gpu!(scope, cond = lhs > 0f32);
    gpu!(&mut scope, if(cond).then(|scope| {
        gpu!(scope, y = lhs + 4.0f32);
    }));

    format!("{:?}", scope.operations)
}

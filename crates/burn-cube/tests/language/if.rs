use burn_cube::{branch::Comptime, cube, Numeric};

#[cube]
pub fn if_greater<T: Numeric>(lhs: T) {
    if lhs > T::from_int(0) {
        let _ = lhs + T::from_int(4);
    }
}

#[cube]
pub fn if_greater_var<T: Numeric>(lhs: T) {
    let x = lhs > T::from_int(0);
    if x {
        let _ = lhs + T::from_int(4);
    }
}


// #[cube]
// pub fn comptime_if<T: Numeric>(lhs: T, comptime: Comptime<bool>) {
//     if Comptime::get(comptime) {
//         let _ = lhs + T::from_int(4);
//     } else {
//         let _ = lhs + T::from_int(5);
//     }
// }

// #[cube]
// pub fn comptime_if<T: Numeric>(lhs: T, comptime: Comptime<Option<(usize, usize)>>) {
//     let (k0, k1, unroll) = if let Some((k0, k1)) = Comptime::get(comptime) {
//         (UInt::new(k0), UInt::new(k1), true)
//     } else {
//         (stride::<T>(input, 0), stride::<T>(input, 1), false)
//     };
// }

mod tests {
    use burn_cube::{
        cpa,
        dialect::{Elem, Item, Variable},
        CubeContext, PrimitiveVariable, F32,
    };

    use super::if_greater_expand;

    type ElemType = F32;

    #[test]
    fn cube_if_test() {
        let mut context = CubeContext::root();

        let lhs = context.create_local(Item::new(ElemType::as_elem()));

        if_greater_expand::<ElemType>(&mut context, lhs);
        let scope = context.into_scope();

        assert_eq!(format!("{:?}", scope.operations), inline_macro_ref());
    }

    fn inline_macro_ref() -> String {
        let mut context = CubeContext::root();
        let item = Item::new(ElemType::as_elem());
        let lhs = context.create_local(item);

        let mut scope = context.into_scope();
        let cond = scope.create_local(Item::new(Elem::Bool));
        let lhs: Variable = lhs.into();
        let y = scope.create_local(item);

        cpa!(scope, cond = lhs > 0f32);
        cpa!(&mut scope, if(cond).then(|scope| {
            cpa!(scope, y = lhs + 4.0f32);
        }));

        format!("{:?}", scope.operations)
    }
}

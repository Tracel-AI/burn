use crate::{kernel::reduce::SumDim, JitElement};
use burn_cube::{
    cpa,
    ir::{Item, Scope, Variable},
};

use super::base::ReduceDimNaive;

impl<E: JitElement> ReduceDimNaive<E> for SumDim {
    type Accumulator = Variable;

    fn initialize_naive(scope: &mut Scope, _input_item: Item, output_item: Item) -> Variable {
        scope.zero(output_item)
    }

    fn inner_loop_naive(scope: &mut Scope, accumulator: Variable, value: Variable, _i: Variable) {
        cpa!(scope, accumulator += value);
    }

    fn assign_naive(
        scope: &mut Scope,
        output: Variable,
        accumulator: Variable,
        _shape_reduce_dim: Variable,
    ) {
        let id = Variable::AbsolutePos;
        cpa!(scope, output[id] = accumulator);
    }
}

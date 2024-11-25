use cubecl::prelude::*;

use crate::kernel::reduce::SumDim;

use super::base::ReduceDimNaive;

#[cube]
impl<EI: Algebraic> ReduceDimNaive<EI> for SumDim {
    type Accumulator = EI;

    fn initialize_naive() -> EI {
        EI::from_int(0)
    }

    fn inner_loop_naive(accumulator: &mut EI, current_value: EI, _i: u32) {
        *accumulator += current_value;
    }

    fn assign_naive<EO: Algebraic>(
        output: &mut Tensor<EO>,
        accumulator: EI,
        _shape_reduce_dim: u32,
    ) {
        output[ABSOLUTE_POS] = EO::cast_from(accumulator);
    }
}

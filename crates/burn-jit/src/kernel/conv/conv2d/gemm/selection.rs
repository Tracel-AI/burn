use super::{
    algorithm::{Algorithm, ImplicitCmmaConv},
    spec::ConvSpec,
};
use crate::JitRuntime;
use cubecl::linalg::matmul::components::{
    stage::CommonStageInput, tile::accelerated::Accelerated, MatmulSelection, MatmulSize,
};

pub struct ConvSelection {
    pub matmul: MatmulSelection,
}

pub trait ConvSelector<A: Algorithm> {
    fn select_kernel<'a, CS: ConvSpec, R: JitRuntime>(plane_dim: u32) -> (A::Selection, A::Input);
}

/// Large m stage size for the usual case where `batch_size * out_h * out_w` is significantly larger
/// than `out_channels`
pub struct Large;
/// Balanced stage size for cases where `batch_size * out_h * out_w` is relatively small and `k` or
/// `out_channels` is relatively large
pub struct Balanced;

impl ConvSelector<ImplicitCmmaConv> for Large {
    fn select_kernel<'a, CS: ConvSpec, R: JitRuntime>(
        plane_dim: u32,
    ) -> (
        <ImplicitCmmaConv as Algorithm>::Selection,
        <ImplicitCmmaConv as Algorithm>::Input,
    ) {
        let selection = MatmulSelection {
            tile: MatmulSize {
                m: 16,
                n: 16,
                k: 16,
            },
            num_stagess: MatmulSize { m: 8, n: 4, k: 2 },
            plane_dim,
        };
        let config_input = CommonStageInput {
            tile: Accelerated::input(selection.tile.clone()),
            num_stages: selection.num_stagess.clone(),
        };

        let selection = ConvSelection { matmul: selection };

        Ok((selection, config_input))
    }
}

impl ConvSelector<ImplicitCmmaConv> for Balanced {
    fn select_kernel<'a, CS: ConvSpec, R: JitRuntime>(
        plane_dim: u32,
    ) -> (
        <ImplicitCmmaConv as Algorithm>::Selection,
        <ImplicitCmmaConv as Algorithm>::Input,
    ) {
        let selection = MatmulSelection {
            tile: MatmulSize {
                m: 16,
                n: 16,
                k: 16,
            },
            num_stagess: MatmulSize { m: 4, n: 2, k: 4 },
            plane_dim,
        };
        let config_input = CommonStageInput {
            tile: Accelerated::input(selection.tile.clone()),
            num_stages: selection.num_stagess.clone(),
        };

        let selection = ConvSelection { matmul: selection };

        Ok((selection, config_input))
    }
}

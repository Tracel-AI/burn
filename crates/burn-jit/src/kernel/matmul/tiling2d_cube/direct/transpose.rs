use burn_cube::prelude::*;

use crate::kernel::matmul::{
    config::CubeTiling2dConfig, tiling2d_cube::load_shared_memory::LoadInfo,
};

use super::{
    loader::{CheckBounds, LoadIndices},
    transpose_trait::TransposeLoad,
};

#[derive(CubeType)]
pub(crate) struct ReadTileInfo {
    read_row: UInt,
    read_col: UInt,
    gm_position_base: UInt,
    sm_position_base: UInt,
    gm_stride: UInt,
    sm_stride: UInt,
}

#[cube]
pub(crate) fn load_transposed<F: Float, S: TransposeLoad<F>>(
    tensor: Tensor<F>,
    load_info: LoadInfo<F>,
    load_indices: LoadIndices,
    check_bounds: CheckBounds,
) {
    let coordinates = load_info.coordinates;
    let config = load_info.config;

    let sm_dim_vertical = Comptime::runtime(Comptime::map(config, |c| c.block_size_k));

    let read_row = coordinates.unit_row;
    let read_col = coordinates.unit_col;
    let write_row = coordinates.unit_col;
    let write_col = coordinates.unit_row;

    let gm_position_base = read_row * load_indices.gm_stride + read_col + load_indices.offset;
    let sm_position_base = write_row * load_indices.sm_stride + write_col;

    let read_tile_info = ReadTileInfo {
        read_row,
        read_col,
        gm_position_base,
        sm_position_base,
        gm_stride: load_indices.gm_stride,
        sm_stride: load_indices.sm_stride,
    };

    if write_row < sm_dim_vertical {
        S::tile_load(
            tensor,
            load_info.shared,
            read_tile_info,
            config,
            check_bounds,
        );
    }
}

#[cube]
pub(crate) fn unchecked_load<F: Float>(
    tensor: Tensor<F>,
    mut shared_memory: SharedMemory<F>,
    info: ReadTileInfo,
    config: Comptime<CubeTiling2dConfig>,
) {
    let tile_size = Comptime::map(config, |c| c.tile_size);
    let unroll = Comptime::map(config, |c| c.unroll_tile);

    for i in range(0u32, Comptime::get(tile_size), unroll) {
        let gm_position = info.gm_position_base + i;
        let sm_position =
            (info.sm_position_base + i * info.sm_stride) / Comptime::runtime(tile_size);

        let mut transposed = F::vectorized_empty(Comptime::get(tile_size));
        for j in range(0u32, Comptime::get(tile_size), unroll) {
            transposed[j] = tensor[gm_position + j * info.gm_stride];
        }

        shared_memory[sm_position] = transposed;
    }
}

#[cube]
pub(crate) fn vertical_check_load<F: Float>(
    tensor: Tensor<F>,
    mut shared_memory: SharedMemory<F>,
    info: ReadTileInfo,
    config: Comptime<CubeTiling2dConfig>,
    check_bounds: CheckBounds,
) {
    let tile_size = Comptime::map(config, |c| c.tile_size);
    let unroll = Comptime::map(config, |c| c.unroll_tile);

    let mut num_reads = UInt::new(0);
    let row = check_bounds.skip_row + info.read_row;
    let dim_vertical = check_bounds.dim_vertical;
    if dim_vertical > row {
        num_reads = UInt::min(dim_vertical - row, Comptime::runtime(tile_size));
    }

    for i in range(0u32, Comptime::get(tile_size), unroll) {
        let gm_position = info.gm_position_base + i;
        let sm_position =
            (info.sm_position_base + i * info.sm_stride) / Comptime::runtime(tile_size);

        let mut transposed = F::vectorized_empty(Comptime::get(tile_size));
        for j in range(0u32, num_reads, Comptime::new(false)) {
            transposed[j] = tensor[gm_position + j * info.gm_stride];
        }
        for j in range(num_reads, Comptime::get(tile_size), Comptime::new(false)) {
            transposed[j] = F::new(0.);
        }

        shared_memory[sm_position] = transposed;
    }
}

#[cube]
pub(crate) fn horizontal_check_load<F: Float>(
    tensor: Tensor<F>,
    mut shared_memory: SharedMemory<F>,
    info: ReadTileInfo,
    config: Comptime<CubeTiling2dConfig>,
    check_bounds: CheckBounds,
) {
    let tile_size = Comptime::map(config, |c| c.tile_size);
    let unroll = Comptime::map(config, |c| c.unroll_tile);

    let mut num_reads = UInt::new(0);
    let col = check_bounds.skip_col + info.read_col;
    let dim_horizontal = check_bounds.dim_horizontal;
    if dim_horizontal > col {
        num_reads = UInt::min(dim_horizontal - col, Comptime::runtime(tile_size));
    }

    for i in range(0u32, num_reads, Comptime::new(false)) {
        let gm_position = info.gm_position_base + i;
        let sm_position =
            (info.sm_position_base + i * info.sm_stride) / Comptime::runtime(tile_size);

        let mut transposed = F::vectorized_empty(Comptime::get(tile_size));
        for j in range(0u32, Comptime::get(tile_size), unroll) {
            transposed[j] = tensor[gm_position + j * info.gm_stride];
        }

        shared_memory[sm_position] = transposed;
    }
}

#[cube]
pub(crate) fn wholly_check_load<F: Float>(
    tensor: Tensor<F>,
    mut shared_memory: SharedMemory<F>,
    info: ReadTileInfo,
    config: Comptime<CubeTiling2dConfig>,
    check_bounds: CheckBounds,
) {
    let tile_size = Comptime::map(config, |c| c.tile_size);
    let unroll = Comptime::map(config, |c| c.unroll_tile);

    let mut num_reads_horizontal = UInt::new(0);
    let col = check_bounds.skip_col + info.read_col;
    let dim_horizontal = check_bounds.dim_horizontal;
    if dim_horizontal > col {
        num_reads_horizontal = UInt::min(dim_horizontal - col, Comptime::runtime(tile_size));
    }

    let mut num_reads_vertical = UInt::new(0);
    let row = check_bounds.skip_row + info.read_row;
    let dim_vertical = check_bounds.dim_vertical;
    if dim_vertical > row {
        num_reads_vertical = UInt::min(dim_vertical - row, Comptime::runtime(tile_size));
    }

    for i in range(0u32, num_reads_horizontal, Comptime::new(false)) {
        let gm_position = info.gm_position_base + i;
        let sm_position =
            (info.sm_position_base + i * info.sm_stride) / Comptime::runtime(tile_size);

        let mut transposed = F::vectorized_empty(Comptime::get(tile_size));
        for j in range(0u32, num_reads_vertical, Comptime::new(false)) {
            transposed[j] = tensor[gm_position + j * info.gm_stride];
        }
        for j in range(
            num_reads_vertical,
            Comptime::get(tile_size),
            Comptime::new(false),
        ) {
            transposed[j] = F::new(0.);
        }

        shared_memory[sm_position] = transposed;
    }
}
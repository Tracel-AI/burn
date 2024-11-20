use cubecl::{linalg::matmul::components::Ident, prelude::*};

use super::Config;

#[derive(CubeType)]
/// A view of a tensor that starts reading data from a specified offset.
/// Ensures safe access by preventing out-of-bounds errors.
/// Includes pre-fetched shapes and strides for optimized performance.
pub struct Im2colReader<E: Numeric> {
    pub tensor: *const Tensor<Line<E>>,
    pub m_offset: u32,
    pub k_offset: u32,

    pub stride_batch: u32,
    pub stride_y: u32,
    pub stride_x: u32,
    pub stride_channel: u32,

    pub shape_batch: u32,
    pub shape_y: u32,
    pub shape_x: u32,
    pub shape_channel: u32,

    pub shape_out_y: u32,
    pub shape_out_x: u32,
}

unsafe impl<E: Numeric> Sync for Im2colReader<E> {}
unsafe impl<E: Numeric> Send for Im2colReader<E> {}

#[cube]
impl<F: Numeric> Im2colReader<F> {
    /// Advance the view along the k dimension by a specified offset, `k_offset`.
    pub fn update_view(&mut self, k_offset: u32) {
        self.k_offset += k_offset;
    }

    /// Reads data from the tensor view at the specified tile coordinates (tile_x, tile_y).
    ///
    /// Each unit loads one line in a coalesced manner for improved efficiency.
    /// For row-major tensors, subsequent units read lines horizontally within the tile,
    /// while for column-major tensors, they read lines vertically.
    ///
    /// # Note
    ///
    /// Out-of-bounds reads will be translated to zeros.
    pub fn load_simple<G: Config>(
        &self,
        tile_x: u32,
        tile_y: u32,
        unit_id: u32,
        #[comptime] ident: Ident,
        #[comptime] config: G,
    ) -> Line<F> {
        let line_size = config.global_line_size(ident);
        let tile_size_x = config.stage_dim(ident).tile_size_x;
        let tile_size_y = config.stage_dim(ident).tile_size_y;

        let view_tile_m = tile_x * tile_size_x + self.m_offset;
        let view_tile_k = tile_y * tile_size_y + self.k_offset;

        let load_m = unit_id / tile_size_y;
        let load_k = unit_id % tile_size_y;

        let view_m = view_tile_m + load_m;
        let view_k = view_tile_k + load_k;

        let out_x = view_m % self.shape_out_x;
        let rem = view_m / self.shape_out_x;
        let out_y = rem % self.shape_out_y;
        let batch = rem / self.shape_out_y;

        let channel = view_k % self.shape_channel;
        let rem = view_k / self.shape_channel;
        let kernel_x = rem % config.kernel_size(0);
        let kernel_y = rem / config.kernel_size(1);

        let y =
            (out_y * config.stride(0) + kernel_y * config.dilation(0)) as i32 - config.padding(0);
        let x =
            (out_x * config.stride(1) + kernel_x * config.dilation(1)) as i32 - config.padding(1);
        let in_bounds = batch < self.shape_batch
            && y >= 0
            && (y as u32) < self.shape_y
            && x >= 0
            && (x as u32) < self.shape_x
            && channel < self.shape_channel;
        let read_pos = batch * self.stride_batch
            + y as u32 * self.stride_y
            + x as u32 * self.stride_x
            + channel * self.shape_channel;

        let read_pos = read_pos / line_size;

        let mut res = Line::empty(line_size).fill(F::from_int(0));
        if config.im2col_unchecked() || in_bounds {
            res = self.read(read_pos);
        }

        res
    }

    fn read(&self, position: u32) -> Line<F> {
        unsafe { *(*self.tensor).index_unchecked(position) }
    }
}

use core::mem::transmute;

use crate::{element::FloatNdArrayElement, tensor::NdArrayTensor, NdArrayBackend, NdArrayDevice};
use burn_tensor::{ops::TensorOps, Shape};
use ndarray::s;
use rayon::prelude::*;

pub(crate) trait Matmul<E> {
    fn matmul<const D: usize>(
        lhs: NdArrayTensor<E, D>,
        rhs: NdArrayTensor<E, D>,
    ) -> NdArrayTensor<E, D>;
}

pub(crate) fn matmul<const D: usize, E: FloatNdArrayElement>(
    lhs: NdArrayTensor<E, D>,
    rhs: NdArrayTensor<E, D>,
) -> NdArrayTensor<E, D> {
    E::matmul(lhs, rhs)
}

impl Matmul<f32> for f32 {
    fn matmul<const D: usize>(
        lhs: NdArrayTensor<f32, D>,
        rhs: NdArrayTensor<f32, D>,
    ) -> NdArrayTensor<f32, D> {
        let shape_ori_lhs = lhs.shape();
        let shape_ori_rhs = rhs.shape();

        let lhs = reshape::<f32, D>(lhs);
        let rhs = reshape::<f32, D>(rhs);

        let [batch_size_lhs, m, _] = lhs.shape().dims;
        let [batch_size_rhs, k, n] = rhs.shape().dims;

        let batch_size = usize::max(batch_size_lhs, batch_size_rhs);

        let out = NdArrayBackend::<f32>::empty(Shape::new([batch_size, m, n]), &NdArrayDevice::Cpu);

        let lhs_strides = lhs.array.strides().to_vec();
        let rhs_strides = rhs.array.strides().to_vec();
        let out_strides = out.array.strides().to_vec();

        let out = matrixmultiply_sgemm(
            m,
            k,
            n,
            lhs,
            &lhs_strides,
            rhs,
            &rhs_strides,
            out,
            &out_strides,
        );

        let mut shape_out = match batch_size_lhs > batch_size_rhs {
            true => shape_ori_lhs,
            false => shape_ori_rhs,
        };
        shape_out.dims[D - 2] = m;
        shape_out.dims[D - 1] = n;

        NdArrayBackend::<f32>::reshape(out, shape_out)
    }
}

struct SharedBuffer<'a> {
    cell: core::cell::UnsafeCell<&'a mut [f32]>,
}

unsafe impl<'a> Sync for SharedBuffer<'a> {}

impl<'a> SharedBuffer<'a> {
    fn new(data: &'a mut [f32]) -> Self {
        Self {
            cell: core::cell::UnsafeCell::new(data),
        }
    }
    fn get(&self) -> &'a mut [f32] {
        unsafe { core::ptr::read(self.cell.get()) }
    }
}

fn matrixmultiply_sgemm(
    m: usize,
    k: usize,
    n: usize,
    lhs: NdArrayTensor<f32, 3>,
    lhs_strides: &[isize],
    rhs: NdArrayTensor<f32, 3>,
    rhs_strides: &[isize],
    mut out: NdArrayTensor<f32, 3>,
    out_strides: &[isize],
) -> NdArrayTensor<f32, 3> {
    let [batch_size_lhs, _, _] = lhs.shape().dims;
    let [batch_size_rhs, _, _] = rhs.shape().dims;
    let [batch_size, _, _] = out.shape().dims;

    if batch_size_lhs > batch_size && batch_size_lhs != 1 {
        panic!("Broadcast on multiple dimensions is not yet supported");
    }

    if batch_size_rhs > batch_size && batch_size_rhs != 1 {
        panic!("Broadcast on multiple dimensions is not yet supported");
    }

    let alpha = 1.0;
    let beta = 0.0;

    let out_slices = out.array.as_slice_mut().unwrap();

    let buffer = SharedBuffer::new(out_slices);

    (0..batch_size).into_par_iter().for_each(|b| {
        let lhs_slice = match batch_size_lhs == 1 {
            true => lhs.array.slice(s!(0, .., ..)),
            false => lhs.array.slice(s!(b, .., ..)),
        };
        let rhs_slice = match batch_size_rhs == 1 {
            true => rhs.array.slice(s!(0, .., ..)),
            false => rhs.array.slice(s!(b, .., ..)),
        };

        let out_buffer = buffer.get();

        unsafe {
            matrixmultiply::sgemm(
                m,
                k,
                n,
                alpha,
                lhs_slice.as_ptr(),
                lhs_strides[1],
                lhs_strides[2],
                rhs_slice.as_ptr(),
                rhs_strides[1],
                rhs_strides[2],
                beta,
                &mut out_buffer[b * (m * n)],
                out_strides[1],
                out_strides[2],
            );
        }
    });

    out
}

impl Matmul<f64> for f64 {
    fn matmul<const D: usize>(
        lhs: NdArrayTensor<f64, D>,
        rhs: NdArrayTensor<f64, D>,
    ) -> NdArrayTensor<f64, D> {
        todo!()
    }
}

fn reshape<E: FloatNdArrayElement, const D: usize>(
    tensor: NdArrayTensor<E, D>,
) -> NdArrayTensor<E, 3> {
    let shape = tensor.shape();

    if D < 2 {
        NdArrayBackend::<E>::reshape(tensor, Shape::new([1, 1, shape.dims[0]]))
    } else {
        let batch_size = batch_size(&shape);
        let size0 = shape.dims[D - 2];
        let size1 = shape.dims[D - 1];

        NdArrayBackend::<E>::reshape(tensor, Shape::new([batch_size, size0, size1]))
    }
}

fn batch_size<const D: usize>(shape: &Shape<D>) -> usize {
    let mut num_batch = 1;
    for i in 0..D - 2 {
        num_batch *= shape.dims[i];
    }

    num_batch
}

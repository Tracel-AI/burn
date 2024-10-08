use crate::{
    ops::{BoolTensor, BoolTensorOps, FloatTensor, IntTensor},
    router::{get_client, BackendRouter, RunnerChannel, RunnerClient},
    Device, Shape,
};

impl<R: RunnerChannel> BoolTensorOps<Self> for BackendRouter<R> {
    fn bool_empty(shape: Shape, device: &Device<Self>) -> BoolTensor<Self> {
        todo!()
    }

    fn bool_shape(tensor: &BoolTensor<Self>) -> Shape {
        todo!()
    }

    fn bool_into_data(
        tensor: BoolTensor<Self>,
    ) -> impl core::future::Future<Output = crate::TensorData> + Send {
        async { tensor.into_data().await }
    }

    fn bool_from_data(data: crate::TensorData, device: &Device<Self>) -> BoolTensor<Self> {
        let client = get_client::<R>(&device);
        client.register_tensor_data(data)
    }

    fn bool_into_int(tensor: BoolTensor<Self>) -> IntTensor<Self> {
        todo!()
    }

    fn bool_into_float(tensor: BoolTensor<Self>) -> FloatTensor<Self> {
        todo!()
    }

    fn bool_device(tensor: &BoolTensor<Self>) -> Device<Self> {
        todo!()
    }

    fn bool_to_device(tensor: BoolTensor<Self>, device: &Device<Self>) -> BoolTensor<Self> {
        todo!()
    }

    fn bool_reshape(tensor: BoolTensor<Self>, shape: Shape) -> BoolTensor<Self> {
        todo!()
    }

    fn bool_slice(
        tensor: BoolTensor<Self>,
        ranges: &[core::ops::Range<usize>],
    ) -> BoolTensor<Self> {
        todo!()
    }

    fn bool_slice_assign(
        tensor: BoolTensor<Self>,
        ranges: &[core::ops::Range<usize>],
        value: BoolTensor<Self>,
    ) -> BoolTensor<Self> {
        todo!()
    }

    fn bool_equal(lhs: BoolTensor<Self>, rhs: BoolTensor<Self>) -> BoolTensor<Self> {
        todo!()
    }

    fn bool_not(tensor: BoolTensor<Self>) -> BoolTensor<Self> {
        todo!()
    }

    fn bool_swap_dims(tensor: BoolTensor<Self>, dim1: usize, dim2: usize) -> BoolTensor<Self> {
        todo!()
    }

    fn bool_permute(tensor: BoolTensor<Self>, axes: &[usize]) -> BoolTensor<Self> {
        todo!()
    }

    fn bool_flip(tensor: BoolTensor<Self>, axes: &[usize]) -> BoolTensor<Self> {
        todo!()
    }

    fn bool_expand(tensor: BoolTensor<Self>, shape: Shape) -> BoolTensor<Self> {
        todo!()
    }
}

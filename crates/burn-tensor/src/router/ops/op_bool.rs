use crate::ops::{BoolTensor, BoolTensorOps, FloatElem, FloatTensor, IntElem, IntTensor};
use crate::repr::{
    BaseOperationDescription, BinaryOperationDescription, BoolOperationDescription,
    ExpandOperationDescription, FlipOperationDescription, OperationDescription,
    PermuteOperationDescription, ReshapeDescription, SliceAssignOperationDescription,
    SliceOperationDescription, SwapDimsDescription, UnaryOperationDescription,
};
use crate::{
    router::{get_client, BackendRouter, RunnerChannel, RunnerClient},
    DType, Device, Element, Shape, TensorData,
};
use alloc::vec::Vec;

impl<R: RunnerChannel> BoolTensorOps<Self> for BackendRouter<R> {
    fn bool_empty(shape: Shape, device: &Device<Self>) -> BoolTensor<Self> {
        // Get the runtime client on which to register the operation for execution.
        let client = get_client::<R>(device);
        client.register_empty_tensor(shape.into(), DType::Bool)
    }

    fn bool_shape(tensor: &BoolTensor<Self>) -> Shape {
        Shape::from(tensor.shape.clone())
    }

    async fn bool_into_data(tensor: BoolTensor<Self>) -> TensorData {
        tensor.into_data().await
    }

    fn bool_from_data(data: crate::TensorData, device: &Device<Self>) -> BoolTensor<Self> {
        let client = get_client::<R>(device);
        client.register_tensor_data(data)
    }

    fn bool_into_int(tensor: BoolTensor<Self>) -> IntTensor<Self> {
        let client = tensor.client.clone();
        let out = client.register_empty_tensor(tensor.shape.clone(), IntElem::<Self>::dtype());

        let desc = UnaryOperationDescription {
            input: tensor.into_description(),
            out: out.to_description_out(),
        };

        client.register(OperationDescription::Bool(
            BoolOperationDescription::IntoInt(desc),
        ));

        out
    }

    fn bool_into_float(tensor: BoolTensor<Self>) -> FloatTensor<Self> {
        let client = tensor.client.clone();
        let out = client.register_empty_tensor(tensor.shape.clone(), FloatElem::<Self>::dtype());

        let desc = UnaryOperationDescription {
            input: tensor.into_description(),
            out: out.to_description_out(),
        };

        client.register(OperationDescription::Bool(
            BoolOperationDescription::IntoFloat(desc),
        ));

        out
    }

    fn bool_device(tensor: &BoolTensor<Self>) -> Device<Self> {
        tensor.client.device()
    }

    fn bool_to_device(tensor: BoolTensor<Self>, device: &Device<Self>) -> BoolTensor<Self> {
        if &tensor.client.device() == device {
            return tensor;
        }
        R::change_client_backend(tensor, device)
    }

    fn bool_reshape(tensor: BoolTensor<Self>, shape: Shape) -> BoolTensor<Self> {
        let client = tensor.client.clone();
        let out = client.register_empty_tensor(shape.into(), tensor.dtype);

        let desc = ReshapeDescription {
            input: tensor.into_description(),
            out: out.to_description_out(),
        };

        client.register(OperationDescription::BaseBool(
            BaseOperationDescription::Reshape(desc),
        ));

        out
    }

    fn bool_slice(
        tensor: BoolTensor<Self>,
        ranges: &[core::ops::Range<usize>],
    ) -> BoolTensor<Self> {
        let client = tensor.client.clone();
        let ndims = tensor.shape().num_dims();
        let mut shape: Vec<usize> = ranges.iter().map(|range| range.end - range.start).collect();

        for i in shape.len()..ndims {
            shape.push(tensor.shape[i]);
        }

        let out = client.register_empty_tensor(shape, tensor.dtype);

        let desc = SliceOperationDescription {
            tensor: tensor.into_description(),
            ranges: ranges.to_vec(),
            out: out.to_description_out(),
        };

        client.register(OperationDescription::BaseBool(
            BaseOperationDescription::Slice(desc),
        ));

        out
    }

    fn bool_slice_assign(
        tensor: BoolTensor<Self>,
        ranges: &[core::ops::Range<usize>],
        value: BoolTensor<Self>,
    ) -> BoolTensor<Self> {
        let client = tensor.client.clone();
        let out = client.register_empty_tensor(tensor.shape.clone(), tensor.dtype);

        let desc = SliceAssignOperationDescription {
            tensor: tensor.into_description(),
            ranges: ranges.to_vec(),
            value: value.into_description(),
            out: out.to_description_out(),
        };

        client.register(OperationDescription::BaseBool(
            BaseOperationDescription::SliceAssign(desc),
        ));

        out
    }

    fn bool_equal(lhs: BoolTensor<Self>, rhs: BoolTensor<Self>) -> BoolTensor<Self> {
        let client = lhs.client.clone();
        let out = client.register_empty_tensor(lhs.shape.clone(), DType::Bool);

        let desc = BinaryOperationDescription {
            lhs: lhs.into_description(),
            rhs: rhs.into_description(),
            out: out.to_description_out(),
        };

        client.register(OperationDescription::BaseBool(
            BaseOperationDescription::Equal(desc),
        ));

        out
    }

    fn bool_not(tensor: BoolTensor<Self>) -> BoolTensor<Self> {
        let client = tensor.client.clone();
        let out = client.register_empty_tensor(tensor.shape.clone(), tensor.dtype);

        let desc = UnaryOperationDescription {
            input: tensor.into_description(),
            out: out.to_description_out(),
        };

        client.register(OperationDescription::Bool(BoolOperationDescription::Not(
            desc,
        )));

        out
    }

    fn bool_swap_dims(tensor: BoolTensor<Self>, dim1: usize, dim2: usize) -> BoolTensor<Self> {
        let client = tensor.client.clone();
        let mut shape = tensor.shape.clone();
        shape[dim1] = tensor.shape[dim2];
        shape[dim2] = tensor.shape[dim1];
        let out = client.register_empty_tensor(shape, tensor.dtype);

        let desc = SwapDimsDescription {
            input: tensor.into_description(),
            out: out.to_description_out(),
            dim1,
            dim2,
        };

        client.register(OperationDescription::BaseBool(
            BaseOperationDescription::SwapDims(desc),
        ));

        out
    }

    fn bool_permute(tensor: BoolTensor<Self>, axes: &[usize]) -> BoolTensor<Self> {
        let client = tensor.client.clone();
        // Change the shape of the tensor to match the new axes
        let shape = axes.iter().map(|x| tensor.shape[*x]).collect();
        let out = client.register_empty_tensor(shape, tensor.dtype);

        let desc = PermuteOperationDescription {
            input: tensor.into_description(),
            out: out.to_description_out(),
            axes: axes.to_vec(),
        };

        client.register(OperationDescription::BaseBool(
            BaseOperationDescription::Permute(desc),
        ));

        out
    }

    fn bool_flip(tensor: BoolTensor<Self>, axes: &[usize]) -> BoolTensor<Self> {
        let client = tensor.client.clone();
        let out = client.register_empty_tensor(tensor.shape.clone(), tensor.dtype);

        let desc = FlipOperationDescription {
            input: tensor.into_description(),
            out: out.to_description_out(),
            axes: axes.to_vec(),
        };

        client.register(OperationDescription::BaseBool(
            BaseOperationDescription::Flip(desc),
        ));

        out
    }

    fn bool_expand(tensor: BoolTensor<Self>, shape: Shape) -> BoolTensor<Self> {
        let client = tensor.client.clone();
        let shape: Vec<_> = shape.into();
        let out = client.register_empty_tensor(shape.clone(), tensor.dtype);

        let desc = ExpandOperationDescription {
            input: tensor.into_description(),
            out: out.to_description_out(),
            shape,
        };

        client.register(OperationDescription::BaseBool(
            BaseOperationDescription::Expand(desc),
        ));

        out
    }
}

use super::reader::NestedValue;

pub trait BurnModuleAdapter: Sized {
    fn adapt(name: &str, data: NestedValue) -> NestedValue {
        match name {
            "BatchNorm" => Self::adapt_batch_norm(data),
            "Conv1d" => Self::adapt_conv1d(data),
            "Conv2d" => Self::adapt_conv2d(data),
            "ConvTranspose1d" => Self::adapt_conv_transpose_1d(data),
            "ConvTranspose2d" => Self::adapt_conv_transpose_2d(data),
            "Embedding" => Self::adapt_embedding(data),
            "GroupNorm" => Self::adapt_group_norm(data),
            "LayerNorm" => Self::adapt_layer_norm(data),
            "Linear" => Self::adapt_linear(data),
            _ => data,
        }
    }

    fn adapt_linear(data: NestedValue) -> NestedValue {
        data
    }

    fn adapt_conv1d(data: NestedValue) -> NestedValue {
        data
    }

    fn adapt_conv2d(data: NestedValue) -> NestedValue {
        data
    }

    fn adapt_conv_transpose_1d(data: NestedValue) -> NestedValue {
        data
    }

    fn adapt_conv_transpose_2d(data: NestedValue) -> NestedValue {
        data
    }

    fn adapt_embedding(data: NestedValue) -> NestedValue {
        data
    }

    fn adapt_group_norm(data: NestedValue) -> NestedValue {
        data
    }

    fn adapt_layer_norm(data: NestedValue) -> NestedValue {
        data
    }

    fn adapt_batch_norm(data: NestedValue) -> NestedValue {
        data
    }
}

/// Default adapter that takes no action.
pub struct DefaultAdapter;
impl BurnModuleAdapter for DefaultAdapter {}

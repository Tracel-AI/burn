use super::{Node, NodeCodegen, SerializationBackend};
use crate::burn::{BurnImports, OtherType, Scope, TensorType, ToTokens, Type};
use burn::{
    module::{ConstantRecord, Param, ParamId},
    nn::conv::{Conv2dConfig, Conv2dRecord},
    record::{PrecisionSettings, Record},
    tensor::{DataSerialize, Tensor},
};
use proc_macro2::TokenStream;
use quote::quote;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Conv2dNode<PS: PrecisionSettings> {
    pub field: OtherType,
    pub input: TensorType,
    pub output: TensorType,
    pub data_weights: DataSerialize<PS::FloatElem>,
    pub data_bias: Option<DataSerialize<PS::FloatElem>>,
    pub config: Conv2dConfig,
}

impl<PS: PrecisionSettings> Conv2dNode<PS> {
    pub fn new<S: AsRef<str>>(
        name: S,
        input: TensorType,
        output: TensorType,
        data_weights: DataSerialize<PS::FloatElem>,
        data_bias: Option<DataSerialize<PS::FloatElem>>,
        config: Conv2dConfig,
    ) -> Self {
        Self {
            field: OtherType::new(
                name,
                quote! {
                    Conv2d<B>
                },
            ),
            input,
            output,
            data_weights,
            data_bias,
            config,
        }
    }
}

impl<PS: PrecisionSettings> Serialize for Conv2dNode<PS> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let record = Conv2dRecord::<SerializationBackend> {
            weight: Param::new(
                ParamId::new(),
                Tensor::from_data(self.data_weights.clone().convert()),
            ),
            bias: self
                .data_bias
                .as_ref()
                .map(|bias| Param::new(ParamId::new(), Tensor::from_data(bias.clone().convert()))),
            stride: [ConstantRecord::new(); 2],
            kernel_size: [ConstantRecord::new(); 2],
            dilation: [ConstantRecord::new(); 2],
            groups: ConstantRecord::new(),
            padding: ConstantRecord::new(),
        };

        let item = Record::into_item::<PS>(record);
        item.serialize(serializer)
    }
}

impl<PS: PrecisionSettings> NodeCodegen<PS> for Conv2dNode<PS> {
    fn input_types(&self) -> Vec<Type> {
        vec![Type::Tensor(&self.input)]
    }
    fn output_types(&self) -> Vec<Type> {
        vec![Type::Tensor(&self.output)]
    }
    fn field_type(&self) -> Option<Type> {
        Some(Type::Other(&self.field))
    }

    fn field_init(&self) -> Option<TokenStream> {
        let name = &self.field.name;
        let channels = self.config.channels.to_tokens();
        let kernel_size = self.config.kernel_size.to_tokens();
        let stride = self.config.stride.to_tokens();
        let dilation = self.config.dilation.to_tokens();
        let groups = self.config.groups.to_tokens();
        let bias = self.config.bias;

        let tokens = quote! {
            let #name = Conv2dConfig::new(#channels, #kernel_size)
                .with_stride(#stride)
                .with_dilation(#dilation)
                .with_groups(#groups)
                .with_bias(#bias)
                .init_with(record.#name);
        };

        Some(tokens)
    }

    fn forward(&self, scope: &mut Scope, node_position: usize) -> TokenStream {
        let input = scope.use_owned_tensor(&self.input.name, node_position);
        let output = &self.output.name;
        let field = &self.field.name;

        quote! {
            let #output = self.#field.forward(#input);
        }
    }
    fn register_imports(&self, imports: &mut BurnImports) {
        imports.register("burn::nn::conv::Conv2d");
        imports.register("burn::nn::conv::Conv2dConfig");
    }

    fn into_node(self) -> Node<PS> {
        Node::Conv2d(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::burn::{
        graph::BurnGraph,
        node::{conv2d::Conv2dNode, test::assert_tokens},
        TensorType,
    };
    use burn::{nn::conv::Conv2dConfig, record::FullPrecisionSettings, tensor::Data};

    #[test]
    fn test_codegen() {
        let mut graph = BurnGraph::<FullPrecisionSettings>::default();

        graph.register(Conv2dNode::new(
            "conv2d",
            TensorType::new("input", 4),
            TensorType::new("output", 4),
            Data::from([2.]).serialize(),
            None,
            Conv2dConfig::new([3, 3], [3, 3]),
        ));

        let expected = quote! {
            use burn::{
                module::Module,
                tensor::{backend::Backend, Tensor},
            };
            use burn::nn::conv::Conv2d;
            use burn::nn::conv::Conv2dConfig;

            #[derive(Module, Debug)]
            pub struct Model <B: Backend> {
                conv2d: Conv2d<B>,
            }

            impl<B: Backend> Model <B> {
                pub fn new_with(record: ModelRecord<B>) -> Self {
                    let conv2d = Conv2dConfig::new([3, 3], [3, 3])
                        .with_stride([1, 1])
                        .with_dilation([1, 1])
                        .with_groups(1)
                        .with_bias(true)
                        .init_with(record.conv2d);

                    Self {
                        conv2d,
                    }
                }

                pub fn forward(&self, input: Tensor<B, 4>) -> Tensor<B, 4> {
                    let output = self.conv2d.forward(input);

                    output
                }
            }
        };

        assert_tokens(graph.codegen(), expected);
    }
}

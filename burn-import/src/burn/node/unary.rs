use super::{Node, NodeCodegen};
use crate::burn::{Scope, TensorType, ToTokens, Type};
use burn::record::PrecisionSettings;
use proc_macro2::TokenStream;
use quote::quote;
use std::sync::Arc;

// Simple fn pointer that receive input as a token stream and return function call.
type FnPointer = Arc<dyn Fn(TokenStream) -> TokenStream>;

/// Node for all unary operators.
#[derive(Clone, new)]
pub struct UnaryNode {
    pub input: TensorType,
    pub output: TensorType,
    pub name: String,
    function: FnPointer,
}

impl std::fmt::Debug for UnaryNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "UnaryNode {{ input: {:?}, output: {:?}, name: {:?} }}",
                self.input, self.output, self.name
            )
            .as_str(),
        )
    }
}

impl<PS: PrecisionSettings> NodeCodegen<PS> for UnaryNode {
    fn output_types(&self) -> Vec<Type> {
        vec![Type::Tensor(&self.output)]
    }

    fn input_types(&self) -> Vec<Type> {
        vec![Type::Tensor(&self.input)]
    }

    fn forward(&self, scope: &mut Scope, node_position: usize) -> TokenStream {
        let input = scope.tensor_use_owned(&self.input, node_position);
        let output = &self.output.name;
        let function = (self.function)(input);

        quote! {
            let #output = #function;
        }
    }

    fn into_node(self) -> Node<PS> {
        Node::Unary(self)
    }
}

impl UnaryNode {
    pub(crate) fn flatten(
        input: TensorType,
        output: TensorType,
        start_dim: usize,
        end_dim: usize,
    ) -> Self {
        let start_dim = start_dim.to_tokens();
        let end_dim = end_dim.to_tokens();
        let function = move |input| quote! { #input.flatten(#start_dim, #end_dim) };

        Self::new(input, output, "flatten".to_string(), Arc::new(function))
    }

    pub(crate) fn relu(input: TensorType, output: TensorType) -> Self {
        let function = move |input| quote! { burn::tensor::activation::relu(#input) };
        Self::new(input, output, "relu".to_string(), Arc::new(function))
    }

    pub(crate) fn sigmoid(input: TensorType, output: TensorType) -> Self {
        let function = move |input| quote! { burn::tensor::activation::sigmoid(#input) };
        Self::new(input, output, "sigmoid".to_string(), Arc::new(function))
    }

    pub(crate) fn log_softmax(input: TensorType, output: TensorType, dim: usize) -> Self {
        let dim = dim.to_tokens();
        let function = move |input| quote! { burn::tensor::activation::log_softmax(#input, #dim) };
        Self::new(input, output, "log_softmax".to_string(), Arc::new(function))
    }
}

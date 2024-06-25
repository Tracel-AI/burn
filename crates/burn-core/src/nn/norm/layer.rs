use crate as burn;

use crate::config::Config;
use crate::module::Module;
use crate::module::Param;
use crate::nn::Initializer;
use crate::tensor::backend::Backend;
use crate::tensor::Tensor;

/// Configuration to create a [LayerNorm](LayerNorm) layer using the [init function](LayerNormConfig::init).
#[derive(Debug, Config)]
pub struct LayerNormConfig {
    /// The size of the input features.
    pub d_model: usize,
    /// A value required for numerical stability. Default: 1e-5
    #[config(default = 1e-5)]
    pub epsilon: f64,
}

/// Applies Layer Normalization over an input tensor as described in the paper [Layer Normalization](https://arxiv.org/abs/1607.06450).
///
/// `Y = norm(X) * γ + β`
///
/// Where:
/// - `X` is the input tensor
/// - `Y` is the output tensor
/// - `γ` is the learnable weight
/// - `β` is the learnable bias
///
/// Should be created using [LayerNormConfig](LayerNormConfig).
#[derive(Module, Debug)]
pub struct LayerNorm<B: Backend> {
    /// The learnable weight.
    gamma: Param<Tensor<B, 1>>,
    /// The learnable bias.
    beta: Param<Tensor<B, 1>>,
    epsilon: f64,
}

impl LayerNormConfig {
    /// Initialize a new [layer norm](LayerNorm) module.
    pub fn init<B: Backend>(&self, device: &B::Device) -> LayerNorm<B> {
        let gamma = Initializer::Ones.init([self.d_model], device);
        let beta = Initializer::Zeros.init([self.d_model], device);

        LayerNorm {
            gamma,
            beta,
            epsilon: self.epsilon,
        }
    }
}

impl<B: Backend> LayerNorm<B> {
    /// Applies the forward pass on the input tensor.
    ///
    /// See the [LayerNorm](LayerNorm) documentation for more information.
    ///
    /// # Shapes
    ///
    /// - input: `[..., any, d_model]`
    /// - output: `[..., any, d_model]`
    pub fn forward<const D: usize>(&self, input: Tensor<B, D>) -> Tensor<B, D> {
        let (var, mean) = input.clone().var_mean_bias(D - 1);

        let input_normalized = input.sub(mean).div(var.sqrt().add_scalar(self.epsilon));

        input_normalized
            .mul(self.gamma.val().unsqueeze())
            .add(self.beta.val().unsqueeze())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tensor::TensorData;

    #[cfg(feature = "std")]
    use crate::{TestAutodiffBackend, TestBackend};

    #[cfg(not(feature = "std"))]
    use crate::TestBackend;

    #[test]
    fn layer_norm_forward() {
        let device = Default::default();
        let module = LayerNormConfig::new(10).init::<TestBackend>(&device);
        let input = Tensor::<TestBackend, 2>::from_data(
            TensorData::from([[
                -0.6897, -2.7106, 2.2222, -1.0330, -0.8933, 1.1765, 0.0601, 1.5252, -0.3630, 0.6728,
            ]]),
            &device,
        );

        let output = module.forward(input);

        let expected = TensorData::from([[
            -0.4990, -1.9680, 1.6178, -0.7486, -0.6470, 0.8576, 0.0461, 1.1111, -0.2614, 0.4915,
        ]]);
        output.to_data().assert_approx_eq(&expected, 3);
    }

    #[cfg(feature = "std")]
    #[test]
    fn layer_norm_backward() {
        let device = Default::default();
        let module = LayerNormConfig::new(2).init::<TestAutodiffBackend>(&device);
        let tensor_1 = Tensor::<TestAutodiffBackend, 2>::from_data(
            TensorData::from([[0.0, 1.0], [3.0, 4.0]]),
            &device,
        )
        .require_grad();
        let tensor_2 = Tensor::<TestAutodiffBackend, 2>::from_data(
            TensorData::from([[6.0, 7.0], [9.0, 10.0]]),
            &device,
        )
        .require_grad();

        let x = tensor_1.clone().matmul(tensor_2.clone());

        let output = module.forward(x);
        let grads = output.backward();

        let tensor_1_grad = tensor_1.grad(&grads).unwrap();
        let tensor_2_grad = tensor_2.grad(&grads).unwrap();
        let gamma_grad = module.gamma.grad(&grads).unwrap();
        let beta_grad = module.beta.grad(&grads).unwrap();

        let expected = TensorData::from([-2.0, 2.0]);
        gamma_grad.to_data().assert_approx_eq(&expected, 3);

        let expected = TensorData::from([2.0, 2.0]);
        beta_grad.to_data().assert_approx_eq(&expected, 3);

        let expected = TensorData::zeros::<f32, _>(tensor_1_grad.shape());
        tensor_1_grad.to_data().assert_approx_eq(&expected, 3);

        let expected = TensorData::zeros::<f32, _>(tensor_2_grad.shape());
        tensor_2_grad.to_data().assert_approx_eq(&expected, 3);
    }
}

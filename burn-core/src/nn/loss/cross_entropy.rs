use core::marker::PhantomData;

use burn_tensor::{activation, backend::Backend, Bool, Int, Tensor};

/// Calculate the cross entropy loss from the input logits and the targets.
#[derive(Clone, Debug, Default)]
pub struct CrossEntropyLoss<B: Backend> {
    pad_index: Option<usize>,
    weights: Option<Tensor<B, 1>>,
    smoothing: Option<f32>,
    backend: PhantomData<B>,
}

impl<B: Backend> CrossEntropyLoss<B> {
    /// Create the criterion.
    pub fn new(pad_index: Option<usize>) -> Self {
        Self {
            pad_index,
            backend: PhantomData,
            smoothing: None,
            weights: None,
        }
    }

    /// Compute the criterion on the input tensor.
    ///
    /// # Shapes
    ///
    /// - logits: `[batch_size, num_targets]`
    /// - targets: `[batch_size]`
    pub fn forward(&self, logists: Tensor<B, 2>, targets: Tensor<B, 1, Int>) -> Tensor<B, 1> {
        if let Some(alpha) = self.smoothing.clone() {
            self.forward_smoothed(logits, targets, alpha)
        } else {
            self.forward_default(logits, targets)
        } 
    }

    fn forward_smoothing(&self, logits: Tensor<B, 2>, targets: Tensor<B, 1, Int>, alpha: f32) -> Tensor<B, 1> {
        let [batch_size] = targets.dims();

        let mask = self.padding_mask(&targets);
        let tensor = activation::log_softmax(logits, 1);
        let scatter_idx = Tensor::cat(vec![Tensor::arange(0..batch_size).unsqueeze(), targets.clone.unsqueeze()], 1);
        let targets_matrix = tensor.zeros_like().scatter(1, scatter_idx, targets); 
        let nr_classes = tensor.dims()[1];
        let target_matrix = target_matrix * (1 - alpha) + alpha / nr_classes;  
        let tensor = tensor * target_matrix;

        if let Some(weights) = self.weights.clone() {
            let tensor = tensor * weights.unsqueeze().repeat(0, batch_size);
            let tensor = tensor.mask_fill(mask.unsqueeze().repeat(0, nr_classes), 0);
            let weights = weights.gather(0, targets);
            (tensor.sum_dim(1) / weights.sum()).neg()
        } else {
            let tensor = tensor.mask_fill(mask.unsqueeze().repeat(0, nr_classes), 0);
            tensor.sum_dim(1).mean().neg()
        }
    } 
    
    fn forward_default(&self, logits: Tensor<B, 2>, targets: Tensor<B, 1, Int>) -> Tensor<B, 1> {
        let [batch_size] = targets.dims();

        let mask = self.padding_mask(&targets);
        let tensor = activation::log_softmax(logits, 1);
        let tensor = tensor.gather(1, targets.clone().reshape([batch_size, 1]));

        if let Some(weights) = self.weights.clone() {
            let weights = weights.gather(0, targets);
            let tensor = tensor.reshape([batch_size]) * weights.clone();
            let tensor = self.apply_mask(tensor, mask);
            tensor.sum().neg() / weights.sum()
        } else {
            let tensor = self.apply_mask(tensor.reshape([batch_size]), mask);
            tensor.mean().neg()
        }
    }

    /// Create weighted cross-entropy. 
    ///
    /// The loss of a specific sample will simply be given by: weight[y] * log(p(x)) * 1, 
    ///
    /// # Pre-conditions
    ///   - The order of the weight vector should correspond to the label integer assignment. 
    ///   - Targets assigned negative Int's will not be allowed.
    pub fn with_weights(self, weights: Vec<f32>) -> Self {
        Self {
            weights: Some(Tensor::<B, 1, Int>::from_floats(weights.as_slice)),
            self..
        }
    }

    /// Create cross-entropy with label smoothing. 
    ///
    /// Hard labels {0, 1} will be changed to y_smoothed = y(1 - a) + a / nr_classes.
    /// Alpah = 0 would be the same as default.
    ///
    pub fn with_smoothing(self, alpha: f32) -> Self {
        assert!(a < 1 && a > 0, "Alpha most be in interval [0, 1]");
        Self {
            weights: Some(alpha),
            self..
        }
    }

    fn padding_mask(&self, targets: &Tensor<B, 1, Int>) -> Option<Tensor<B, 1, Bool>> {
        let mut mask = None;
        if let Some(pad_index) = self.pad_index {
            mask = Some(targets.clone().equal_elem(pad_index as i64));
        }

        mask
    }

    fn apply_mask(
        &self,
        mut tensor: Tensor<B, 1>,
        mask: Option<Tensor<B, 1, Bool>>,
    ) -> Tensor<B, 1> {
        if let Some(mask) = mask {
            tensor = tensor.mask_fill(mask, 0);
        }

        tensor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestBackend;
    use burn_tensor::{loss::cross_entropy_with_logits, Data, Distribution};

    #[test]
    fn test_cross_entropy_loss_with_weights() {
        let [batch_size, num_targets] = [4, 5];
        let logits = Tensor::<TestBackend, 2>::random(
            [batch_size, num_targets],
            Distribution::Normal(0., 1.0),
        );
        let weights = vec![1.0, 2., 3., 4., 5.];
        let targets = Tensor::<TestBackend, 1, Int>::from_data(Data::from([2, 0, 4, 1]));
        let targets_logits = Tensor::<TestBackend, 2>::from_data(Data::from([
            [0.0, 0.0, 1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 0.0, 0.0],
        ]));

        let loss_1 = CrossEntropyLoss::new(None)
            .with_weights(weights.clone())
            .forward(logits.clone(), targets);
        let tensor = activation::log_softmax(logits, 1);
        let loss_2 = tensor
            * targets_logits
            * Tensor::<TestBackend, 1>::from_floats(weights.as_slice())
                .unsqueeze()
                .repeat(0, 4);
        let loss_2 = loss_2.sum().neg() / (1. + 2. + 3. + 5.);
        loss_1.into_data().assert_approx_eq(&loss_2.into_data(), 3);
    }

    #[test]
    fn test_cross_entropy_loss() {
        let [batch_size, num_targets] = [4, 5];
        let logits = Tensor::<TestBackend, 2>::random(
            [batch_size, num_targets],
            Distribution::Normal(0., 1.0),
        );
        let targets = Tensor::<TestBackend, 1, Int>::from_data(Data::from([2, 0, 4, 1]));
        let targets_logits = Tensor::<TestBackend, 2>::from_data(Data::from([
            [0.0, 0.0, 1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 0.0, 0.0],
        ]));

        let loss_1 = CrossEntropyLoss::new(None).forward(logits.clone(), targets);
        let loss_2 = cross_entropy_with_logits(logits, targets_logits);

        loss_1.into_data().assert_approx_eq(&loss_2.into_data(), 3);
    }

    #[test]
    fn test_cross_entropy_loss_with_pad_token() {
        let [batch_size, num_targets, pad_index] = [4, 5, 1];
        let logits = Tensor::<TestBackend, 2>::random(
            [batch_size, num_targets],
            Distribution::Normal(0., 1.0),
        );
        let targets = Tensor::<TestBackend, 1, Int>::from_data(
            Data::<i64, 1>::from([2, 0, 4, pad_index as i64]).convert(),
        );
        let targets_logits = Tensor::<TestBackend, 2>::from_data(Data::from([
            [0.0, 0.0, 1.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0, 0.0, 0.0],
        ]));

        let loss_1 = CrossEntropyLoss::new(Some(pad_index)).forward(logits.clone(), targets);
        let loss_2 = cross_entropy_with_logits(logits, targets_logits);

        loss_1.into_data().assert_approx_eq(&loss_2.into_data(), 3);
    }
}

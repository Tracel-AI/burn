use std::fmt::Display;

use burn_compute::tune::AutotuneKey;

use crate::kernel::{matmul::MatmulAutotuneKey, reduce::ReduceAutotuneKey};

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
/// Key for all autotune-enabled operations
pub enum WgpuAutotuneKey {
    /// Key for matmul operation
    Matmul(MatmulAutotuneKey),
    /// Key for reduce operations
    Reduce(ReduceAutotuneKey),
}

impl Display for WgpuAutotuneKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WgpuAutotuneKey::Matmul(matmul_key) => std::fmt::Display::fmt(&matmul_key, f),
            WgpuAutotuneKey::Reduce(reduce_key) => std::fmt::Display::fmt(&reduce_key, f),
        }
    }
}

impl AutotuneKey for WgpuAutotuneKey {}

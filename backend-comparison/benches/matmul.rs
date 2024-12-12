use backend_comparison::persistence::save;
use burn::tensor::{activation::gelu, backend::Backend, Shape, Tensor};
use burn_common::benchmark::{run_benchmark, Benchmark};
use derive_new::new;

#[derive(new)]
struct MatmulBenchmark<B: Backend, const D: usize> {
    shape_lhs: Shape,
    shape_rhs: Shape,
    device: B::Device,
}

impl<B: Backend, const D: usize> Benchmark for MatmulBenchmark<B, D> {
    type Args = (Tensor<B, D>, Tensor<B, D>, Tensor<B, D>);

    fn name(&self) -> String {
        "matmul".into()
    }

    fn shapes(&self) -> Vec<Vec<usize>> {
        vec![self.shape_lhs.dims.clone(), self.shape_rhs.dims.clone()]
    }

    fn execute(&self, (lhs, rhs, bias): Self::Args) {
        let b = lhs.matmul(rhs) + bias;
        gelu(b);
    }

    fn prepare(&self) -> Self::Args {
        let lhs = Tensor::zeros(self.shape_lhs.clone(), &self.device);
        let rhs = Tensor::zeros(self.shape_rhs.clone(), &self.device);
        let bias = Tensor::zeros(self.shape_rhs.clone(), &self.device);

        (lhs, rhs, bias)
    }

    fn sync(&self) {
        B::sync(&self.device)
    }
}

#[allow(dead_code)]
fn bench<B: Backend>(
    device: &B::Device,
    feature_name: &str,
    url: Option<&str>,
    token: Option<&str>,
) {
    let benchmarks = [
        // (3, 4096, 4096, 4096),
        (8, 2048, 2048, 2048),
        // (2, 4096, 4096, 512),
    ]
    .into_iter()
    .map(|(b, m, n, k)| {
        let shape_lhs = [b, m, k].into();
        let shape_rhs = [b, k, n].into();

        MatmulBenchmark::<B, 3>::new(shape_lhs, shape_rhs, device.clone())
    })
    .map(run_benchmark)
    .collect();

    save::<B>(benchmarks, device, feature_name, url, token).unwrap();
}

fn main() {
    backend_comparison::bench_on_backend!();
}

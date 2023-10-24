use alloc::sync::Arc;
use burn_common::benchmark::Benchmark;

use crate::server::{ComputeServer, Handle};

use super::AutotuneOperation;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// A benchmark that runs on server handles
#[derive(new)]
pub struct TuneBenchmark<'a, S: ComputeServer> {
    operation: Arc<dyn AutotuneOperation<S>>,
    handles: Vec<Handle<S>>,
    server: &'a mut S,
}

impl<'a, S: ComputeServer> Benchmark for TuneBenchmark<'a, S> {
    type Args = ();

    fn prepare(&self) -> Self::Args {}

    fn num_samples(&self) -> usize {
        1
    }

    fn execute(&mut self, _: Self::Args) {
        self.operation.execute(
            &self.handles.iter().collect::<Vec<&Handle<S>>>(),
            self.server,
        )
    }

    fn name(&self) -> String {
        "Autotune".to_string()
    }

    fn sync(&mut self) {
        self.server.sync();
    }
}

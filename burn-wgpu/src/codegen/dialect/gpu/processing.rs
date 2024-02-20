use crate::codegen::dialect::gpu::ReadGlobalWithLayout;

use super::{Operation, Procedure, Variable};

/// Information necessary when compiling a scope.
pub struct ScopeProcessing {
    /// The variable declarations.
    pub variables: Vec<Variable>,
    /// The operations.
    pub operations: Vec<Operation>,
}

impl ScopeProcessing {
    /// Optimize the [variables](Variable) and [operations](Operation).
    ///
    /// ## Notes:
    ///
    /// This should be called once right after the creation of the type.
    /// If you built this type from the [scope process function](super::Scope::process), you don't have to
    /// call it again.
    pub fn optimize(self) -> Self {
        self.merge_read_global_with_layout()
    }

    /// Merge all compatible [read global with layout procedures](ReadGlobalWithLayout).
    fn merge_read_global_with_layout(mut self) -> Self {
        #[derive(Default)]
        struct Optimization {
            merged_procs: Vec<MergedProc>,
        }

        #[derive(new)]
        struct MergedProc {
            proc: ReadGlobalWithLayout,
            positions: Vec<usize>,
        }

        impl Optimization {
            fn register(&mut self, proc: &ReadGlobalWithLayout, position: usize) {
                for merged_proc in self.merged_procs.iter_mut() {
                    if let Some(merged) = merged_proc.proc.try_merge(proc) {
                        merged_proc.proc = merged;
                        merged_proc.positions.push(position);
                        return;
                    }
                }

                self.merged_procs
                    .push(MergedProc::new(proc.clone(), vec![position]));
            }
        }

        let mut optimization = Optimization::default();

        for (position, operation) in self.operations.iter().enumerate() {
            let proc = match operation {
                Operation::Procedure(algo) => algo,
                _ => continue,
            };

            let proc = match proc {
                Procedure::ReadGlobalWithLayout(algo) => algo,
                _ => continue,
            };

            optimization.register(proc, position);
        }

        if optimization.merged_procs.is_empty() {
            return self;
        }

        let mut operations = Vec::with_capacity(self.operations.len());

        for (position, operation) in self.operations.into_iter().enumerate() {
            let mut is_merged_op = false;

            for merged_proc in optimization.merged_procs.iter() {
                if merged_proc.positions[0] == position {
                    operations.push(Operation::Procedure(Procedure::ReadGlobalWithLayout(
                        merged_proc.proc.clone(),
                    )));
                    is_merged_op = true;
                }

                if merged_proc.positions.contains(&position) {
                    is_merged_op = true;
                }
            }

            if !is_merged_op {
                operations.push(operation);
            }
        }

        self.operations = operations;
        self
    }
}

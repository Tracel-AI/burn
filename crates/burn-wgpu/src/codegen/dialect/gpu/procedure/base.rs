use super::{
    ConditionalAssign, Gather, IndexOffsetGlobalWithLayout, Matmul, ReadGlobal,
    ReadGlobalWithLayout, WriteGlobal,
};
use crate::codegen::dialect::gpu::Vectorization;
use serde::{Deserialize, Serialize};

/// Tensor operations that can't be executed with a simple [operator](super::super::Operator) should use a
/// procedure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Procedure {
    ReadGlobalWithLayout(ReadGlobalWithLayout),
    IndexOffsetGlobalWithLayout(IndexOffsetGlobalWithLayout),
    ReadGlobal(ReadGlobal),
    Matmul(Matmul),
    WriteGlobal(WriteGlobal),
    ConditionalAssign(ConditionalAssign),
    Gather(Gather),
}

impl Procedure {
    pub fn vectorize(&self, vectorization: Vectorization) -> Self {
        match self {
            Procedure::ReadGlobalWithLayout(op) => {
                Procedure::ReadGlobalWithLayout(op.vectorize(vectorization))
            }
            Procedure::ReadGlobal(op) => Procedure::ReadGlobal(op.vectorize(vectorization)),
            Procedure::Matmul(op) => Procedure::Matmul(op.vectorize(vectorization)),
            Procedure::WriteGlobal(op) => Procedure::WriteGlobal(op.vectorize(vectorization)),
            Procedure::ConditionalAssign(proc) => {
                Procedure::ConditionalAssign(proc.vectorize(vectorization))
            }
            Procedure::IndexOffsetGlobalWithLayout(op) => {
                Procedure::IndexOffsetGlobalWithLayout(op.vectorize(vectorization))
            }
            Procedure::Gather(proc) => Procedure::Gather(proc.vectorize(vectorization)),
        }
    }
}

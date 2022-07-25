use super::{BackwardNodeState, ForwardNodeRef, Ones, Zeros};
use crate::{
    grad::Gradients,
    ops::{
        BackwardRecordedOpsRef, Forward2BackwardGraphConverter, RecordedOpsParent,
        RecordedOpsParentRef,
    },
};
use std::{collections::HashSet, ops::Add, sync::Arc};

#[derive(Debug)]
pub struct BackwardNode<Out> {
    pub id: String,
    pub order: usize,
    pub state: BackwardNodeState<Out>,
    pub ops: BackwardRecordedOpsRef<Out>,
}
pub type BackwardNodeRef<Out> = Arc<BackwardNode<Out>>;

impl<Out: Clone + Zeros<Out>> BackwardNode<Out> {
    pub fn from_node(
        node: &ForwardNodeRef<Out>,
        converter: &mut Forward2BackwardGraphConverter,
    ) -> Self {
        BackwardNode {
            id: node.id.clone(),
            order: node.order,
            state: BackwardNodeState::new(node.state.value()),
            ops: node.ops.as_backward(converter),
        }
    }
}

impl<Out> BackwardNode<Out>
where
    Out: Zeros<Out> + Ones<Out> + Clone + Add<Output = Out>,
    Out: std::fmt::Debug + 'static,
{
    pub fn backward(&mut self) -> Gradients {
        let grad = self.state.value().ones();
        self.state.update_grad(grad);
        self.ops.backward_step(&mut self.state);

        let mut visited = HashSet::with_capacity(self.order);
        visited.insert(self.id.clone());

        let mut nodes = Vec::with_capacity(self.order);
        for _ in 0..self.order + 1 {
            nodes.push(Vec::new());
        }

        let mut parents = self.ops.backward_parents();

        loop {
            match parents.pop() {
                Some(node) => {
                    let id = node.id();
                    let order = node.order();

                    if order == 0 {
                        continue;
                    }

                    for parent in node.backward_parents() {
                        let id = parent.id();

                        if !visited.contains(id) {
                            parents.push(parent);
                        }
                    }
                    match nodes.get_mut(order) {
                        Some(nodes) => {
                            if !visited.contains(id) {
                                visited.insert(id.clone());
                                nodes.push(node);
                            }
                        }
                        None => {}
                    };
                }
                None => break,
            }
        }

        for i in (0..self.order + 1).rev() {
            if let Some(nodes) = nodes.get(i) {
                for node in nodes {
                    node.backward_step();
                }
            }
        }

        Gradients::from(&self)
    }
}

impl<T> RecordedOpsParent for BackwardNode<T>
where
    T: Zeros<T> + Clone + Add<Output = T>,
    T: std::fmt::Debug + 'static,
{
    fn backward_step(&self) {
        println!("backward node id={} order={}", self.id, self.order);
        self.ops.backward_step(&self.state)
    }
    fn backward_parents(&self) -> Vec<RecordedOpsParentRef> {
        self.ops.backward_parents()
    }

    fn order(&self) -> usize {
        self.order
    }
    fn id(&self) -> &String {
        &self.id
    }
    fn register_grad(&self, grads: &mut Gradients) {
        grads.register(&self)
    }
}

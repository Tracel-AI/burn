use crate::{
    data::dataloader::DataLoaderIterator,
    module::ADModule,
    optim::visitor::convert_grads_to_param,
    train::{TrainOutput, TrainStep},
};
use burn_tensor::backend::ADBackend;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::spawn;

pub struct MultiDevicesTrainStep<B: ADBackend, M, TI, TO> {
    workers: Vec<Worker<B, M, TI>>,
    receiver: Receiver<TrainOutput<B, TO>>,
}

struct Message<M, TI> {
    item: TI,
    model: M,
}

struct Worker<B: ADBackend, M, TI> {
    sender_input: Sender<Message<M, TI>>,
    device: B::Device,
}

impl<B: ADBackend, M, TI> Worker<B, M, TI>
where
    M: ADModule<ADBackend = B> + Clone,
{
    fn register(&self, item: TI, model: &M) {
        let message = Message {
            item,
            model: model.clone(),
        };
        self.sender_input.send(message).unwrap();
    }

    fn start<TO>(
        &self,
        sender_output: Sender<TrainOutput<B, TO>>,
        receiver_input: Receiver<Message<M, TI>>,
    ) where
        TI: Send + 'static,
        TO: Send + 'static,
        M: TrainStep<B, TI, TO> + Send + 'static,
    {
        let device = self.device.clone();

        spawn(move || loop {
            match receiver_input.recv() {
                Ok(item) => {
                    let mut step = item.model;
                    step.to_device(device.clone());
                    step.detach();

                    let mut output = step.step(item.item);
                    convert_grads_to_param(&mut output.grads, &step);
                    sender_output.send(output).unwrap();
                }
                Err(_err) => break,
            };
        });
    }
}

impl<B, M, TI, TO> MultiDevicesTrainStep<B, M, TI, TO>
where
    B: ADBackend,
    TI: Send + 'static,
    TO: Send + 'static,
    M: ADModule<ADBackend = B> + TrainStep<B, TI, TO> + Send + Clone + 'static,
{
    pub fn new(devices: &[B::Device]) -> Self
    where
        TI: Send + 'static,
    {
        let (sender_output, receiver_output) = std::sync::mpsc::channel();
        let workers = devices
            .iter()
            .map(|device| {
                let (sender_input, receiver_input) = std::sync::mpsc::channel();
                let worker = Worker {
                    sender_input,
                    device: device.clone(),
                };

                worker.start(sender_output.clone(), receiver_input);
                worker
            })
            .collect();

        Self {
            workers,
            receiver: receiver_output,
        }
    }

    pub fn step<'a>(
        &self,
        dataloader: &mut Box<dyn DataLoaderIterator<TI> + 'a>,
        model: &M,
    ) -> Vec<TrainOutput<B, TO>> {
        let mut num_send = 0;

        for worker in self.workers.iter() {
            if let Some(item) = dataloader.next() {
                worker.register(item, model);
                num_send += 1;
            }
        }

        let mut outputs = Vec::with_capacity(num_send);

        for _ in 0..num_send {
            let output = self.receiver.recv().unwrap();
            outputs.push(output);
        }

        outputs
    }
}

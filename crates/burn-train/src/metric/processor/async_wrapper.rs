use super::{Event, EventProcessor};
use std::sync::mpsc::{Receiver, Sender};

pub struct AsyncProcessor<P: EventProcessor> {
    sender: Sender<Message<P>>,
}

struct Worker<P: EventProcessor> {
    processor: P,
    rec: Receiver<Message<P>>,
}

impl<P: EventProcessor + 'static> Worker<P> {
    pub fn start(mut processor: P, rec: Receiver<Message<P>>) {
        std::thread::spawn(move || {
            while let Ok(msg) = rec.recv() {
                match msg {
                    Message::Train(event) => processor.process_train(event),
                    Message::Valid(event) => processor.process_valid(event),
                }
            }
        });
    }
}

impl<P: EventProcessor + 'static> AsyncProcessor<P> {
    pub fn new(processor: P) -> Self {
        let (sender, rec) = std::sync::mpsc::channel();

        Worker::start(processor, rec);

        Self { sender }
    }
}

enum Message<P: EventProcessor> {
    Train(Event<P::ItemTrain>),
    Valid(Event<P::ItemValid>),
}

impl<P: EventProcessor> EventProcessor for AsyncProcessor<P> {
    type ItemTrain = P::ItemTrain;
    type ItemValid = P::ItemValid;

    fn process_train(&mut self, event: Event<Self::ItemTrain>) {
        self.sender.send(Message::Train(event)).unwrap();
    }

    fn process_valid(&mut self, event: Event<Self::ItemValid>) {
        self.sender.send(Message::Valid(event)).unwrap();
    }
}

use burn_core::{data::dataloader::Progress, LearningRate};

use crate::info::MetricsUpdate;

pub enum Event {
    /// Signal that an item have been processed.
    MetricsUpdate(MetricsUpdate),
    /// Signal the end of an epoch.
    EndEpoch(usize),
}

/// Defines how training and validation events are collected and searched.
///
/// This trait also exposes methods that uses the collected data to compute useful information.
pub trait EventStore: Send {
    /// Collect the training event.
    fn add_event(&mut self, event: Event, split: Split);

    /// Find the epoch following the given criteria from the collected data.
    fn find_epoch(
        &mut self,
        name: &str,
        aggregate: Aggregate,
        direction: Direction,
        split: Split,
    ) -> Option<usize>;

    /// Find the metric value for the current epoch following the given criteria.
    fn find_metric(
        &mut self,
        name: &str,
        epoch: usize,
        aggregate: Aggregate,
        split: Split,
    ) -> Option<f64>;
}

#[derive(Copy, Clone)]
/// How to aggregate the metric.
pub enum Aggregate {
    /// Compute the average.
    Mean,
}

#[derive(Copy, Clone)]
/// The split to use.
pub enum Split {
    /// The training split.
    Train,
    /// The validation split.
    Valid,
}

#[derive(Copy, Clone)]
/// The direction of the query.
pub enum Direction {
    /// Lower is better.
    Lowest,
    /// Higher is better.
    Highest,
}

/// A learner item.
#[derive(new)]
pub struct LearnerItem<T> {
    /// The item.
    pub item: T,

    /// The progress.
    pub progress: Progress,

    /// The epoch.
    pub epoch: usize,

    /// The total number of epochs.
    pub epoch_total: usize,

    /// The iteration.
    pub iteration: usize,

    /// The learning rate.
    pub lr: Option<LearningRate>,
}

// #[cfg(test)]
// pub mod test_utils {
//     use crate::{info::MetricsInfo, Aggregate, Direction, Event, EventStore, Split};
//
//     #[derive(new)]
//     pub struct TestEventCollector<T, V>
//     where
//         T: Send + Sync + 'static,
//         V: Send + Sync + 'static,
//     {
//         info: MetricsInfo<T, V>,
//     }
//
//     impl<T, V> EventStore for TestEventCollector<T, V>
//     where
//         T: Send + Sync + 'static,
//         V: Send + Sync + 'static,
//     {
//         type ItemTrain = T;
//         type ItemValid = V;
//
//         fn add_event_train(&mut self, event: Event<Self::ItemTrain>) {
//             match event {
//                 Event::ProcessedItem(item) => {
//                     let metadata = (&item).into();
//                     self.info.update_train(&item, &metadata);
//                 }
//                 Event::EndEpoch(epoch) => self.info.end_epoch_train(epoch),
//             }
//         }
//
//         fn add_event_valid(&mut self, event: Event<Self::ItemValid>) {
//             match event {
//                 Event::ProcessedItem(item) => {
//                     let metadata = (&item).into();
//                     self.info.update_valid(&item, &metadata);
//                 }
//                 Event::EndEpoch(epoch) => self.info.end_epoch_valid(epoch),
//             }
//         }
//
//         fn find_epoch(
//             &mut self,
//             name: &str,
//             aggregate: Aggregate,
//             direction: Direction,
//             split: Split,
//         ) -> Option<usize> {
//             self.info.find_epoch(name, aggregate, direction, split)
//         }
//
//         fn find_metric(
//             &mut self,
//             name: &str,
//             epoch: usize,
//             aggregate: Aggregate,
//             split: Split,
//         ) -> Option<f64> {
//             self.info.find_metric(name, epoch, aggregate, split)
//         }
//     }
// }

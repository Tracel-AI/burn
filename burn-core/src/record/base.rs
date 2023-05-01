pub use burn_derive::Record;

use super::PrecisionSettings;
use serde::{de::DeserializeOwned, Serialize};

/// Trait to define a family of types which can be recorded using any [settings](RecordSettings).
pub trait Record: Send + Sync {
    type Item<S: PrecisionSettings>: Serialize + DeserializeOwned;

    /// Convert the current record into the corresponding item that follows the given [settings](RecordSettings).
    fn into_item<S: PrecisionSettings>(self) -> Self::Item<S>;
    /// Convert the given item into a record.
    fn from_item<S: PrecisionSettings>(item: Self::Item<S>) -> Self;
}

use super::{Record, RecordSettings};
use crate::module::{Param, ParamId, State};
use alloc::vec::Vec;
use burn_tensor::{DataSerialize, Element};
use hashbrown::HashMap;

impl Record for () {
    type Item<S: RecordSettings> = ();

    fn into_item<S: RecordSettings>(self) -> Self::Item<S> {}

    fn from_item<S: RecordSettings>(_item: Self::Item<S>) -> Self {}
}

impl<T: Record> Record for Vec<T> {
    type Item<S: RecordSettings> = Vec<T::Item<S>>;

    fn into_item<S: RecordSettings>(self) -> Self::Item<S> {
        self.into_iter().map(Record::into_item).collect()
    }

    fn from_item<S: RecordSettings>(item: Self::Item<S>) -> Self {
        item.into_iter().map(Record::from_item).collect()
    }
}

impl<T: Record> Record for Option<T> {
    type Item<S: RecordSettings> = Option<T::Item<S>>;

    fn into_item<S: RecordSettings>(self) -> Self::Item<S> {
        self.map(Record::into_item)
    }

    fn from_item<S: RecordSettings>(item: Self::Item<S>) -> Self {
        item.map(Record::from_item)
    }
}

impl<const N: usize, T: Record> Record for [T; N] {
    type Item<S: RecordSettings> = [T::Item<S>; N];

    fn into_item<S: RecordSettings>(self) -> Self::Item<S> {
        self.map(Record::into_item)
    }

    fn from_item<S: RecordSettings>(item: Self::Item<S>) -> Self {
        item.map(Record::from_item)
    }
}

impl<T: Record> Record for HashMap<ParamId, T> {
    type Item<S: RecordSettings> = HashMap<ParamId, T::Item<S>>;

    fn into_item<S: RecordSettings>(self) -> Self::Item<S> {
        let mut items = HashMap::with_capacity(self.len());
        self.into_iter().for_each(|(id, record)| {
            items.insert(id, record.into_item());
        });
        items
    }

    fn from_item<S: RecordSettings>(item: Self::Item<S>) -> Self {
        let mut record = HashMap::with_capacity(item.len());
        item.into_iter().for_each(|(id, item)| {
            record.insert(id, T::from_item(item));
        });
        record
    }
}

impl<T: Element> Record for State<T> {
    type Item<S: RecordSettings> = State<S::FloatElem>;

    fn into_item<S: RecordSettings>(self) -> Self::Item<S> {
        self.convert::<S::FloatElem>()
    }

    fn from_item<S: RecordSettings>(item: Self::Item<S>) -> Self {
        item.convert()
    }
}
impl<E: Element> Record for DataSerialize<E> {
    type Item<S: RecordSettings> = DataSerialize<S::FloatElem>;

    fn into_item<S: RecordSettings>(self) -> Self::Item<S> {
        self.convert()
    }

    fn from_item<S: RecordSettings>(item: Self::Item<S>) -> Self {
        item.convert()
    }
}

impl<T: Record> Record for Param<T> {
    type Item<S: RecordSettings> = Param<T::Item<S>>;

    fn into_item<S: RecordSettings>(self) -> Self::Item<S> {
        Param {
            id: self.id,
            value: self.value.into_item(),
        }
    }

    fn from_item<S: RecordSettings>(item: Self::Item<S>) -> Self {
        Param {
            id: item.id,
            value: T::from_item(item.value),
        }
    }
}

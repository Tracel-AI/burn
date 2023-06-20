use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use serde::de::DeserializeOwned;

use crate::Dataset;

/// Dataset where all items are stored in ram.
pub struct InMemDataset<I> {
    items: Vec<I>,
}

impl<I> InMemDataset<I> {
    pub fn new(items: Vec<I>) -> Self {
        InMemDataset { items }
    }
}

impl<I> Dataset<I> for InMemDataset<I>
where
    I: Clone + Send + Sync,
{
    fn get(&self, index: usize) -> Option<I> {
        self.items.get(index).cloned()
    }
    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<I> InMemDataset<I>
where
    I: Clone + DeserializeOwned,
{
    /// Create from a dataset. All items are loaded in memory.
    pub fn from_dataset(dataset: &impl Dataset<I>) -> Self {
        let items: Vec<I> = dataset.iter().collect();
        Self::new(items)
    }

    /// Create from a json rows file (one json per line).
    ///
    /// [Supported field types](https://docs.rs/serde_json/latest/serde_json/value/enum.Value.html)
    pub fn from_json_rows<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut items = Vec::new();

        for line in reader.lines() {
            let item = serde_json::from_str(line.unwrap().as_str()).unwrap();
            items.push(item);
        }

        let dataset = Self::new(items);

        Ok(dataset)
    }

    /// Create from a csv file.
    ///
    /// The first line of the csv file must be the header. The header must contain the name of the fields in the struct.
    ///
    /// The supported field types are: String, integer, float, and bool.
    ///
    /// See: [Reading with Serde](https://docs.rs/csv/latest/csv/tutorial/index.html#reading-with-serde)
    pub fn from_csv<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut rdr = csv::Reader::from_reader(reader);

        let mut items = Vec::new();

        for result in rdr.deserialize() {
            let item: I = result?;
            items.push(item);
        }

        let dataset = Self::new(items);

        Ok(dataset)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{test_data, SqliteDataset};

    use rstest::{fixture, rstest};
    use serde::{Deserialize, Serialize};

    const DB_FILE: &str = "tests/data/sqlite-dataset.db";
    const JSON_FILE: &str = "tests/data/dataset.json";
    const CSV_FILE: &str = "tests/data/dataset.csv";

    type SqlDs = SqliteDataset<Sample>;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct Sample {
        column_str: String,
        column_bytes: Vec<u8>,
        column_int: i64,
        column_bool: bool,
        column_float: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct SampleCvs {
        column_str: String,
        column_int: i64,
        column_bool: bool,
        column_float: f64,
    }

    #[fixture]
    fn train_dataset() -> SqlDs {
        SqliteDataset::from_db_file(DB_FILE, "train").unwrap()
    }

    #[rstest]
    pub fn from_dataset(train_dataset: SqlDs) {
        let dataset = InMemDataset::from_dataset(&train_dataset);

        let non_existing_record_index: usize = 10;
        let record_index: usize = 0;

        assert_eq!(train_dataset.get(non_existing_record_index), None);
        assert_eq!(dataset.get(record_index).unwrap().column_str, "HI1");
    }

    #[test]
    pub fn from_json_rows() {
        let dataset = InMemDataset::<Sample>::from_json_rows(JSON_FILE).unwrap();

        let non_existing_record_index: usize = 10;
        let record_index: usize = 1;

        assert_eq!(dataset.get(non_existing_record_index), None);
        assert_eq!(dataset.get(record_index).unwrap().column_str, "HI2");
        assert!(!dataset.get(record_index).unwrap().column_bool);
    }

    #[test]
    pub fn from_csv_rows() {
        let dataset = InMemDataset::<SampleCvs>::from_csv(CSV_FILE).unwrap();

        let non_existing_record_index: usize = 10;
        let record_index: usize = 1;

        assert_eq!(dataset.get(non_existing_record_index), None);
        assert_eq!(dataset.get(record_index).unwrap().column_str, "HI2");
        assert_eq!(dataset.get(record_index).unwrap().column_int, 1);
        assert!(!dataset.get(record_index).unwrap().column_bool);
        assert_eq!(dataset.get(record_index).unwrap().column_float, 1.0);
    }

    #[test]
    pub fn given_in_memory_dataset_when_iterate_should_iterate_though_all_items() {
        let items_original = test_data::string_items();
        let dataset = InMemDataset::new(items_original.clone());

        let items: Vec<String> = dataset.iter().collect();

        assert_eq!(items_original, items);
    }
}

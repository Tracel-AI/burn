use super::downloader::cache_dir;
use crate::source::huggingface::downloader::{download, Extractor};
use crate::{Dataset, InMemDataset};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MNISTItem {
    pub image: [[f32; 28]; 28],
    pub label: usize,
}

pub struct MNISTDataset {
    dataset: InMemDataset<MNISTItem>,
}

impl Dataset<MNISTItem> for MNISTDataset {
    fn get(&self, index: usize) -> Option<MNISTItem> {
        self.dataset.get(index)
    }

    fn len(&self) -> usize {
        self.dataset.len()
    }
}

impl MNISTDataset {
    pub fn train() -> Self {
        Self::new("train")
    }
    pub fn test() -> Self {
        Self::new("test")
    }

    fn new(split: &str) -> Self {
        let cache_dir = cache_dir();
        let path_file = format!("{}/mnist-{}", cache_dir, split);

        if !std::path::Path::new(path_file.as_str()).exists() {
            download(
                "mnist".to_string(),
                vec![split.to_string()],
                "mnist".to_string(),
                vec![
                    Extractor::Image("image".to_string()),
                    Extractor::Raw("label".to_string()),
                ],
                vec![],
                vec![],
            );
        }
        let dataset = InMemDataset::from_file(path_file.as_str()).unwrap();

        Self { dataset }
    }
}

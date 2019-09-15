use chrono;
use serde::{Deserialize, Serialize};
use serde_json;

use util;

use std::error::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BasicMeta {
    // TODO: replace with u32
    hash: Option<i64>,
    timestamp: i64,
}

impl BasicMeta {
    pub fn new() -> BasicMeta {
        let now = chrono::Utc::now();
        BasicMeta {
            hash: None,
            timestamp: now.timestamp(),
        }
    }

    pub fn with_timestamp(self, timestamp: i64) -> Self {
        BasicMeta {
            hash: self.hash,
            timestamp,
        }
    }

    pub fn with_compute_hash<T>(self, obj: &T) -> Result<Self, Box<dyn Error + Send + Sync>>
    where
        T: Serialize + ?Sized,
    {
        let json = serde_json::to_string(&obj)?;
        Ok(BasicMeta {
            timestamp: self.timestamp,
            hash: Some(i64::from(util::compute_hash(&json))),
        })
    }

    pub fn with_hash(self, hash_value: i64) -> Self {
        BasicMeta {
            hash: Some(hash_value),
            timestamp: self.timestamp,
        }
    }

    pub fn get_hash(&self) -> Option<i64> {
        self.hash
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::offset::TimeZone;

    #[test]
    fn create() {
        let _ = BasicMeta::new();
    }

    #[test]
    fn create_with_hash() {
        let _ = BasicMeta::new().with_hash(123);
    }

    #[test]
    fn create_with_timestamp() {
        let _ = BasicMeta::new().with_timestamp(123);
    }

    #[test]
    fn get_hash_none() {
        let meta = BasicMeta::new();
        assert!(meta.get_hash().is_none());
    }

    #[test]
    fn get_hash_some() {
        let meta = BasicMeta::new().with_hash(123);
        assert!(meta.get_hash().is_some());
        assert_eq!(meta.get_hash(), Some(123));
    }

    #[test]
    fn get_timestamp() {
        let meta = BasicMeta::new();
        let dr = chrono::Utc.timestamp(meta.get_timestamp(), 0);
        assert_ne!(dr.timestamp(), 0);
    }

    #[test]
    fn set_timestamp() {
        let metadata = BasicMeta::new().with_timestamp(1234);
        assert_eq!(1234, metadata.get_timestamp());
    }

    #[test]
    fn with_compute_hash() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let metadata = BasicMeta::new().with_compute_hash(&vec).unwrap();
        assert!(metadata.get_hash().unwrap() >= 0);
    }

    #[test]
    fn with_compute_hash_is_the_same() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let metadata_1 = BasicMeta::new().with_compute_hash(&vec).unwrap();
        let metadata_2 = BasicMeta::new().with_compute_hash(&vec).unwrap();
        assert_eq!(metadata_1.get_hash(), metadata_2.get_hash());
    }

    #[test]
    fn with_compute_hash_different_hash() {
        let vec_1 = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let vec_2 = vec![11, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let metadata_1 = BasicMeta::new().with_compute_hash(&vec_1).unwrap();
        let metadata_2 = BasicMeta::new().with_compute_hash(&vec_2).unwrap();
        assert_ne!(metadata_1.get_hash(), metadata_2.get_hash());
    }

    #[test]
    fn with_compute_hash_multiple_times_is_the_same() {
        let vec = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let metadata = BasicMeta::new().with_compute_hash(&vec).unwrap();
        let first_hash = metadata.get_hash();
        let second_hash = metadata.get_hash();
        assert_eq!(first_hash, second_hash);
    }
}

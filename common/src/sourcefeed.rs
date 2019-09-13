use serde::{Deserialize, Serialize};
use serde_json;

use util;

use std::error::Error;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SourceFeed {
    // TODO: make private and use getter
    pub url: String,
    title: String,
    // TODO: change back to u32 after using new datastore
    hash: Option<i64>,
}

impl SourceFeed {
    fn new_no_hash(url: &str, title: &str) -> SourceFeed {
        SourceFeed {
            url: url.to_string(),
            title: title.to_string(),
            hash: None,
        }
    }

    pub fn new(url: &str, title: &str) -> Result<SourceFeed, Box<dyn Error>> {
        SourceFeed::new_no_hash(url, title).with_compute_hash()
    }

    fn with_compute_hash(self) -> Result<Self, Box<dyn Error>> {
        let json = serde_json::to_string(&self)?;

        Ok(SourceFeed {
            url: self.url.clone(),
            title: self.title.clone(),
            hash: Some(i64::from(util::compute_hash(&json))),
        })
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    pub fn get_hash(&self) -> i64 {
        match self.hash {
            Some(v) => v,
            // this case should not happened unless creating struct with no hash is exposed
            None => self
                .clone()
                .with_compute_hash()
                .expect("internal hashing error")
                .hash
                .unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE1: &str = "test1";
    const SOURCE2: &str = "test2";

    #[test]
    fn new() {
        let _ = SourceFeed::new(SOURCE1, "").unwrap();
    }

    #[test]
    fn get_title() {
        let title = "this is title";
        let source_feed = SourceFeed::new(SOURCE1, title).unwrap();
        assert_eq!(title.to_string(), source_feed.get_title());
    }

    #[test]
    fn get_url() {
        let source_feed = SourceFeed::new(SOURCE1, "").unwrap();
        assert_eq!(SOURCE1.to_string(), source_feed.get_url());
    }

    #[test]
    fn generate_hash() {
        let source_feed = SourceFeed::new(SOURCE1, "").unwrap();
        assert!(source_feed.hash.is_some());
        assert!(source_feed.get_hash() >= 0);
    }

    #[test]
    fn generate_hash_is_the_same() {
        let source_feed_1 = SourceFeed::new(SOURCE1, "").unwrap();
        let source_feed_2 = SourceFeed::new(SOURCE1, "").unwrap();
        assert!(source_feed_1.hash.is_some());
        assert!(source_feed_2.hash.is_some());
        assert_eq!(source_feed_1.get_hash(), source_feed_2.get_hash());
    }

    #[test]
    fn generate_different_hash() {
        let source_feed_1 = SourceFeed::new(SOURCE1, "").unwrap();
        let source_feed_2 = SourceFeed::new(SOURCE2, "").unwrap();
        assert!(source_feed_1.hash.is_some());
        assert!(source_feed_2.hash.is_some());
        assert_ne!(source_feed_1.get_hash(), source_feed_2.get_hash());
    }

    #[test]
    fn get_hash_multiple_times_is_the_same() {
        let source_feed = SourceFeed::new(SOURCE1, "").unwrap();
        assert!(source_feed.hash.is_some());
        let first_hash = source_feed.get_hash();
        let second_hash = source_feed.get_hash();
        assert_eq!(first_hash, second_hash);
    }
}

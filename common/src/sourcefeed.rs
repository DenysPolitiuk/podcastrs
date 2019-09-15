use serde::{Deserialize, Serialize};

use super::BasicMeta;

use std::error::Error;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SourceFeed {
    url: String,
    title: String,
    metadata: BasicMeta,
}

impl SourceFeed {
    fn new_no_hash(url: &str, title: &str) -> SourceFeed {
        SourceFeed {
            url: url.to_string(),
            title: title.to_string(),
            metadata: BasicMeta::new(),
        }
    }

    pub fn new(url: &str, title: &str) -> Result<SourceFeed, Box<dyn Error + Send + Sync>> {
        SourceFeed::new_no_hash(url, title).with_compute_hash()
    }

    fn with_compute_hash(self) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let meta = self.metadata.clone();
        Ok(SourceFeed {
            url: self.url.clone(),
            title: self.title.clone(),
            metadata: meta.with_compute_hash(&self)?,
        })
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn get_url(&self) -> String {
        self.url.clone()
    }

    pub fn get_hash(&self) -> i64 {
        match self.metadata.get_hash() {
            Some(v) => v,
            // this case should not happened unless creating struct with no hash is exposed
            None => self
                .clone()
                .with_compute_hash()
                .expect("internal hashing error")
                .get_hash(),
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
    fn get_hash() {
        let source_feed = SourceFeed::new(SOURCE1, "").unwrap();
        assert!(source_feed.get_hash() >= 0);
    }
}

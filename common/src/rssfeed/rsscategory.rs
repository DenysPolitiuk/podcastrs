use rss::Category;
use serde::{Deserialize, Serialize};

use super::super::BasicMeta;

use std::error::Error;

#[derive(Clone, Deserialize, Serialize)]
pub struct RssCategory {
    name: String,
    domain: Option<String>,
    metadata: BasicMeta,
}

impl RssCategory {
    fn new_no_hash(category: &Category) -> RssCategory {
        RssCategory {
            name: category.name().to_string(),
            domain: category.domain().map(|s| s.to_string()),
            metadata: BasicMeta::new(),
        }
    }

    pub fn new_from_category(category: &Category) -> Result<RssCategory, Box<dyn Error>> {
        RssCategory::new_no_hash(category).with_compute_hash()
    }

    fn with_compute_hash(self) -> Result<Self, Box<dyn Error>> {
        let meta = self.metadata.clone();

        Ok(RssCategory {
            name: self.name.clone(),
            domain: self.domain.clone(),
            metadata: meta.with_compute_hash(&self)?,
        })
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_domain(&self) -> Option<String> {
        self.domain.clone()
    }

    fn get_timestamp(&self) -> i64 {
        self.metadata.get_timestamp()
    }

    pub fn get_hash(&self) -> i64 {
        match self.metadata.get_hash() {
            Some(v) => v,
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

    use rss::Channel;

    use std::fs::File;
    use std::io::BufReader;

    static TEST_FEED: &str = "../tests/sedaily.rss";
    static TEST_FEED_2: &str = "../tests/sedaily2.rss";

    fn create_channel_from_file(name: &str) -> Channel {
        let file = File::open(name).unwrap();
        let buf_reader = BufReader::new(&file);
        Channel::read_from(buf_reader).unwrap()
    }

    fn create_catergory(feed_name: &str) -> RssCategory {
        let channel = create_channel_from_file(feed_name);
        RssCategory::new_from_category(&channel.items()[0].categories()[0]).unwrap()
    }

    #[test]
    fn create() {
        let _ = create_catergory(TEST_FEED);
    }

    #[test]
    fn get_name() {
        let mut category = create_catergory(TEST_FEED);
        let text = "this is name".to_string();
        category.name = text.clone();
        assert_eq!(category.get_name(), text);
    }

    #[test]
    fn get_domain_none() {
        let mut category = create_catergory(TEST_FEED);
        category.domain = None;
        assert!(category.get_domain().is_none());
    }

    #[test]
    fn get_domain_some() {
        let mut category = create_catergory(TEST_FEED);
        let text = Some("this is domain".to_string());
        category.domain = text.clone();
        assert!(category.get_domain().is_some());
        assert_eq!(category.get_domain(), text);
    }

    #[test]
    fn get_timestamp() {
        let mut category = create_catergory(TEST_FEED);
        category.metadata = category.metadata.with_timestamp(123);
        assert_eq!(category.get_timestamp(), 123);
    }

    #[test]
    fn get_hash() {
        let mut category = create_catergory(TEST_FEED);
        category.metadata = category.metadata.with_hash(123);
        assert_eq!(category.get_hash(), 123);
    }
}

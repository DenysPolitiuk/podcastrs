use reqwest;
use rss::Channel;
use serde::{Deserialize, Serialize};

use super::super::BasicMeta;
use super::RssCategory;
use super::RssFeedCore;
use super::RssItem;

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

#[derive(Clone, Deserialize, Serialize)]
pub struct RssFeed {
    core: Option<RssFeedCore>,
    items: Option<Vec<RssItem>>,
    categories: Option<Vec<RssCategory>>,
    metadata: BasicMeta,
    source_feed_url: String,
    feed_file_location: String,
}

impl RssFeed {
    pub fn new_from_url(
        source_url: &str,
        file_to_save: &str,
    ) -> Result<RssFeed, Box<dyn Error + Send + Sync>> {
        let mut f = File::create(&file_to_save)?;
        let _ = reqwest::get(source_url)?.copy_to(&mut f);

        let result = RssFeed::new_from_file(source_url, &file_to_save);

        result
    }

    pub fn new_from_file(
        source_url: &str,
        file_name: &str,
    ) -> Result<RssFeed, Box<dyn Error + Send + Sync>> {
        let file = File::open(&file_name)?;
        let buf_reader = BufReader::new(&file);
        let channel = Channel::read_from(buf_reader)?;

        Ok(RssFeed {
            source_feed_url: source_url.to_string(),
            feed_file_location: String::from(file_name),
            core: Some(RssFeedCore::new_from_channel(&channel)?),
            items: Some(
                channel
                    .items()
                    .iter()
                    .map(|i| RssItem::new_from_item(i))
                    .filter_map(|i| i.ok())
                    .collect(),
            ),
            categories: Some(
                channel
                    .categories()
                    .iter()
                    .map(|c| RssCategory::new_from_category(c))
                    .filter_map(|c| c.ok())
                    .collect(),
            ),
            metadata: BasicMeta::new(),
        }
        .with_compute_hash()?)
    }

    fn with_compute_hash(self) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let meta = self.metadata.clone();

        Ok(RssFeed {
            source_feed_url: self.source_feed_url.clone(),
            feed_file_location: self.feed_file_location.clone(),
            core: self.core.clone(),
            items: self.items.clone(),
            categories: self.categories.clone(),
            metadata: meta.with_compute_hash(&self)?,
        })
    }

    pub fn get_source_feed(&self) -> &String {
        &self.source_feed_url
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

    pub fn get_file_location(&self) -> String {
        self.feed_file_location.clone()
    }

    pub fn get_items(&self) -> Option<&[RssItem]> {
        match &self.items {
            None => None,
            Some(items) => Some(&items),
        }
    }

    pub fn save_item_to_file<P: AsRef<Path>>(
        item: &RssItem,
        file_name: P,
    ) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_name)?;

        RssFeed::save_item(&item, &mut BufWriter::new(file))?;
        Ok(())
    }

    pub fn save_item<W: Write>(item: &RssItem, writer: &mut W) -> Result<(), Box<dyn Error>> {
        let enclosure_url = match item.get_enclosure_url() {
            None => Err("unable to get enclosure for the item")?,
            Some(v) => v,
        };

        let _ = reqwest::get(enclosure_url)?.copy_to(writer);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::NamedTempFile;

    static TEST_FEED: &str = "../tests/sedaily.rss";
    static TEST_FEED_2: &str = "../tests/sedaily2.rss";
    static TEST_SOURCE_FEED_URL: &str = "test";
    static REAL_FEED_URL: &str = "https://softwareengineeringdaily.com/category/podcast/feed";

    fn test_feed() -> RssFeed {
        match RssFeed::new_from_file(TEST_SOURCE_FEED_URL, TEST_FEED) {
            Ok(v) => v,
            Err(e) => {
                println!("{}", e);
                panic!(e)
            }
        }
    }

    #[test]
    fn get_hash() {
        let mut feed = test_feed();
        feed.metadata = feed.metadata.with_hash(1234);
        assert_eq!(feed.get_hash(), 1234);
    }

    #[test]
    fn create_feed_from_file() {
        let feed = test_feed();

        let items = feed.get_items().unwrap();
        let mut valid_items = 0;
        for item in items {
            let _ = match item.get_title() {
                None => continue,
                Some(v) => v,
            };
            let _ = match item.get_enclosure_url() {
                None => continue,
                Some(v) => v,
            };
            valid_items += 1;
        }

        assert_ne!(feed.get_items().unwrap().len(), 0);
        assert_eq!(valid_items, feed.get_items().unwrap().len());
    }

    #[test]
    #[ignore]
    fn create_feed_from_url() {
        let temp_file = NamedTempFile::new().unwrap();
        let feed =
            RssFeed::new_from_url(REAL_FEED_URL, temp_file.path().to_str().unwrap()).unwrap();

        assert!(!feed.get_items().unwrap().is_empty());
        assert_ne!(feed.get_file_location(), "".to_string());

        assert!(temp_file.as_file().metadata().unwrap().is_file());
        assert_ne!(temp_file.as_file().metadata().unwrap().len(), 0);
    }

    #[test]
    #[ignore]
    fn save_item() {
        let feed = test_feed();
        let items = feed.get_items().unwrap();
        let item = &items[0];
        let temp_file = NamedTempFile::new().unwrap();

        RssFeed::save_item_to_file(&item, temp_file.path()).unwrap();

        assert_ne!(temp_file.as_file().metadata().unwrap().len(), 0);
    }
}

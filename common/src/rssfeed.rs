use reqwest;
use rss::{Channel, Item};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

#[derive(Clone, Deserialize, Serialize)]
pub struct RssFeed {
    source_feed_url: String,
    channel: Channel,
    // TODO: better datatype ?
    hash: String,
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

        let file = File::open(&file_name)?;
        let mut buf_reader = BufReader::new(&file);
        let mut buffer = vec![];
        buf_reader.read_to_end(&mut buffer)?;
        let mut hasher = Sha512::new();
        hasher.input(&buffer);
        let hash = hasher.result();

        Ok(RssFeed {
            source_feed_url: source_url.to_string(),
            channel,
            hash: format!("{:x}", hash),
            feed_file_location: String::from(file_name),
        })
    }

    pub fn get_source_feed(&self) -> &String {
        &self.source_feed_url
    }

    pub fn get_hash(&self) -> &str {
        self.hash.as_str()
    }

    pub fn get_file_location(&self) -> String {
        self.feed_file_location.clone()
    }

    pub fn get_items(&self) -> &[Item] {
        self.channel.items()
    }

    pub fn save_item_to_file<P: AsRef<Path>>(
        item: &Item,
        file_name: P,
    ) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_name)?;

        RssFeed::save_item(&item, &mut BufWriter::new(file))?;
        Ok(())
    }

    pub fn save_item<W: Write>(item: &Item, writer: &mut W) -> Result<(), Box<dyn Error>> {
        let enclosure = match item.enclosure() {
            None => Err("unable to get enclosure for the item")?,
            Some(v) => v,
        };

        let _ = reqwest::get(enclosure.url())?.copy_to(writer);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::NamedTempFile;

    static TEST_FEED: &str = "../tests/sedaily.rss";
    static TEST_FEED_HASH: &str = "bbebeae954a00d0426239111a5d632b366073736abaa04e080c49b280b7622c23c0e2485e4701acf77b5b541f14a34421dfb1c905e3191b15837d056950a8d8f";
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
    fn create_feed_has_hash() {
        let feed = test_feed();

        assert_ne!(feed.hash, String::new());
        assert_eq!(feed.hash.as_str(), TEST_FEED_HASH);
    }

    #[test]
    fn create_feed_from_file() {
        let feed = test_feed();

        let items = feed.get_items();
        let mut valid_items = 0;
        for item in items {
            let _ = match item.title() {
                None => continue,
                Some(v) => v,
            };
            let _ = match item.enclosure() {
                None => continue,
                Some(v) => v,
            };
            valid_items += 1;
        }

        assert_ne!(feed.get_items().len(), 0);
        assert_eq!(valid_items, feed.get_items().len());
    }

    #[test]
    #[ignore]
    fn create_feed_from_url() {
        let temp_file = NamedTempFile::new().unwrap();
        let feed =
            RssFeed::new_from_url(REAL_FEED_URL, temp_file.path().to_str().unwrap()).unwrap();

        assert!(!feed.get_items().is_empty());
        assert_ne!(feed.get_file_location(), "".to_string());

        assert!(temp_file.as_file().metadata().unwrap().is_file());
        assert_ne!(temp_file.as_file().metadata().unwrap().len(), 0);
    }

    #[test]
    #[ignore]
    fn save_item() {
        let feed = test_feed();
        let items = feed.get_items();
        let item = &items[0];
        let temp_file = NamedTempFile::new().unwrap();

        RssFeed::save_item_to_file(&item, temp_file.path()).unwrap();

        assert_ne!(temp_file.as_file().metadata().unwrap().len(), 0);
    }
}

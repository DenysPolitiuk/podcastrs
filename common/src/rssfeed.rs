use reqwest;
use rss::{Channel, Enclosure, Guid, Item};
use serde::{Deserialize, Serialize};
use serde_json;

use util;

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

#[derive(Clone, Deserialize, Serialize)]
pub struct MiniChannel {
    title: String,
    description: String,
    link: String,
    last_build_date: Option<String>,
    items: Vec<MiniItem>,
}

impl MiniChannel {
    pub fn from_channel(channel: &Channel) -> MiniChannel {
        MiniChannel {
            title: channel.title().to_string(),
            description: channel.description().to_string(),
            link: channel.link().to_string(),
            last_build_date: channel.last_build_date().map(|s| s.to_string()),
            items: channel
                .items()
                .iter()
                .map(|i| MiniItem::from_item(i))
                .collect(),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct MiniItem {
    title: Option<String>,
    link: Option<String>,
    pub_date: Option<String>,
    guid: Option<Guid>,
    enclosure: Option<Enclosure>,
    description: Option<String>,
}

impl MiniItem {
    pub fn from_item(item: &Item) -> MiniItem {
        MiniItem {
            title: item.title().map(|s| s.to_string()),
            link: item.link().map(|s| s.to_string()),
            pub_date: item.pub_date().map(|s| s.to_string()),
            guid: item.guid().cloned(),
            enclosure: item.enclosure().cloned(),
            description: item.description().map(|s| s.to_string()),
        }
    }

    pub fn get_title(&self) -> Option<&str> {
        self.title.as_ref().map(String::as_ref)
    }

    pub fn get_guid(&self) -> Option<&str> {
        self.guid.as_ref().map(|g| &*g.value())
    }

    pub fn get_enclosure_url(&self) -> Option<&str> {
        self.enclosure.as_ref().map(|e| &*e.url())
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RssFeed {
    source_feed_url: String,
    #[serde(skip)]
    channel: Option<Channel>,
    mini_channel: MiniChannel,
    // TODO: better datatype ?
    // TODO: replace with u32, similar to SourceFeed
    hash: Option<i64>,
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
            hash: None,
            feed_file_location: String::from(file_name),
            mini_channel: MiniChannel::from_channel(&channel),
            channel: Some(channel),
        }
        .with_compute_hash()?)
    }

    fn with_compute_hash(self) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let json = serde_json::to_string(&self)?;

        Ok(RssFeed {
            source_feed_url: self.source_feed_url.clone(),
            hash: Some(i64::from(util::compute_hash(&json))),
            feed_file_location: self.feed_file_location.clone(),
            mini_channel: self.mini_channel.clone(),
            channel: self.channel.clone(),
        })
    }

    pub fn get_source_feed(&self) -> &String {
        &self.source_feed_url
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

    pub fn get_file_location(&self) -> String {
        self.feed_file_location.clone()
    }

    pub fn get_items(&self) -> &[MiniItem] {
        &self.mini_channel.items
    }

    pub fn save_item_to_file<P: AsRef<Path>>(
        item: &MiniItem,
        file_name: P,
    ) -> Result<(), Box<dyn Error>> {
        let file = File::create(file_name)?;

        RssFeed::save_item(&item, &mut BufWriter::new(file))?;
        Ok(())
    }

    pub fn save_item<W: Write>(item: &MiniItem, writer: &mut W) -> Result<(), Box<dyn Error>> {
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

        assert_ne!(feed.get_hash(), 0);
    }

    #[test]
    fn create_feed_from_file() {
        let feed = test_feed();

        let items = feed.get_items();
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

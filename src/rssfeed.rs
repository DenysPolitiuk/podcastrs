use md5;
use reqwest;
use rss::{Channel, Item};

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};

pub struct RssFeed {
    channel: Channel,
    // TODO: better datatype ?
    hash: String,
}

impl RssFeed {
    pub fn new_from_file(file_name: &str) -> Result<RssFeed, Box<Error + Send + Sync>> {
        let file = File::open(file_name)?;
        let buf_reader = BufReader::new(&file);
        let channel = Channel::read_from(buf_reader)?;

        let file = File::open(file_name)?;
        let mut buf_reader = BufReader::new(&file);
        let mut buffer = vec![];
        buf_reader.read_to_end(&mut buffer)?;
        let hash = md5::compute(buffer);

        Ok(RssFeed {
            channel,
            hash: format!("{:x}", hash),
        })
    }

    pub fn get_hash(&self) -> &str {
        self.hash.as_str()
    }

    pub fn get_items(&self) -> &[Item] {
        self.channel.items()
    }

    // TODO: change name to save_item_to_file
    pub fn save_to_file(item: &Item, file_name: &str) -> Result<(), Box<dyn Error>> {
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

    static TEST_FEED: &str = "tests/sedaily.rss";
    static TEST_FEED_HASH: &str = "04dd6b58dccc7944162a934948df3da3";

    fn test_feed() -> RssFeed {
        match RssFeed::new_from_file(TEST_FEED) {
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
    fn save_item() {
        let feed = test_feed();
        let items = feed.get_items();
        let item = &items[0];

        // TODO: instead of using item.title() should use temp file from `tempdir` crate
        RssFeed::save_to_file(&item, item.title().or(Some("test.mp3")).unwrap()).unwrap();
    }
}

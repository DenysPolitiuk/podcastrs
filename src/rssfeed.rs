use reqwest;
use rss::{Channel, Item};

use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};

pub struct RssFeed {
    channel: Channel,
}

impl RssFeed {
    pub fn new_from_file(file_name: &str) -> Result<RssFeed, Box<dyn Error>> {
        let file = File::open(file_name)?;
        let channel = Channel::read_from(BufReader::new(file))?;

        Ok(RssFeed { channel })
    }

    pub fn get_items(&self) -> &[Item] {
        self.channel.items()
    }

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

    fn test_feed() -> RssFeed {
        RssFeed::new_from_file("sedaily.rss").unwrap()
    }

    #[test]
    fn create_feed_from_file() {
        let feed = test_feed();

        let items = feed.get_items();
        let mut valid_items = 0;
        for item in items {
            let title = match item.title() {
                None => continue,
                Some(v) => v,
            };
            let enclosure = match item.enclosure() {
                None => continue,
                Some(v) => v,
            };
            valid_items += 1;
            println!("{} @ {}", title, enclosure.url());
        }

        println!("total number of items is {}", feed.get_items().len());

        assert_ne!(feed.get_items().len(), 0);
        assert_eq!(valid_items, feed.get_items().len());
    }

    #[test]
    fn save_item() {
        let feed = test_feed();
        let items = feed.get_items();
        let item = &items[0];

        RssFeed::save_to_file(&item, item.title().or(Some("test.mp3")).unwrap()).unwrap();
    }
}

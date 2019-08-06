use crate::RssFeed;
use crate::SourceFeed;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub struct RssScheduler {
    source_feeds: HashMap<String, SourceFeed>,
    rss_feeds: HashMap<String, Vec<RssFeed>>,
}

impl RssScheduler {
    pub fn new() -> RssScheduler {
        RssScheduler {
            source_feeds: HashMap::new(),
            rss_feeds: HashMap::new(),
        }
    }

    pub fn add_source_feed(&mut self, source_feed_url: &str) -> bool {
        let url = source_feed_url.to_string();
        match self.source_feeds.entry(url) {
            Entry::Occupied(_) => false,
            Entry::Vacant(v) => {
                v.insert(SourceFeed::new(source_feed_url));
                true
            }
        }
    }

    pub fn get_source_feed(&self, url: &str) -> Option<&SourceFeed> {
        self.source_feeds.get(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_source_feed() {
        let mut scheduler = RssScheduler::new();

        assert_eq!(scheduler.source_feeds.len(), 0);
        assert_eq!(scheduler.add_source_feed("test1"), true);
        assert_eq!(scheduler.add_source_feed("test2"), true);
        assert_eq!(scheduler.add_source_feed("test2"), false);
        assert_eq!(scheduler.add_source_feed("test3"), true);
        assert_eq!(scheduler.source_feeds.len(), 3);
    }

    #[test]
    fn get_source_feeds() {
        const SOURCE1: &str = "test1";
        const SOURCE2: &str = "test2";
        const SOURCE3: &str = "test3";
        const INVALID_SOURCE: &str = "invalid";

        let mut scheduler = RssScheduler::new();
        scheduler.add_source_feed(SOURCE1);
        scheduler.add_source_feed(SOURCE2);
        scheduler.add_source_feed(SOURCE3);

        assert_eq!(scheduler.get_source_feed(SOURCE1).unwrap().url, SOURCE1);
        assert_eq!(scheduler.get_source_feed(SOURCE2).unwrap().url, SOURCE2);
        assert_eq!(scheduler.get_source_feed(SOURCE3).unwrap().url, SOURCE3);
        assert!(scheduler.get_source_feed(INVALID_SOURCE).is_none());
    }

    #[test]
    fn get_new_feed_from_feed_source() {
        panic!("unimplemented");
    }

    #[test]
    fn check_feed_update() {
        panic!("unimplemented");
    }

    #[test]
    fn update_feed() {
        panic!("unimplemented");
    }
}

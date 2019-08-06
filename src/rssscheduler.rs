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
        panic!("unimplemented");
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

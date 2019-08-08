use crate::RssFeed;
use crate::SourceFeed;

use std::collections::hash_map::Entry;
use std::collections::HashMap;

pub trait RssSchedulerStorage {
    fn get_source_feeds(&self) -> HashMap<String, SourceFeed>;
    fn get_last_rss_feed(&self, url: &str) -> Option<RssFeed>;
    fn add_new_rss_feed(&mut self, feed: RssFeed);
    fn add_source_feed(&mut self, source_feed: SourceFeed);
}

pub struct RssScheduler {
    source_feeds: HashMap<String, SourceFeed>,
    rss_feeds: HashMap<String, RssFeed>,
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

    pub fn add_new_feed(&mut self, new_feed: RssFeed) -> bool {
        let source_url = new_feed.get_source_feed();
        match self.rss_feeds.entry(source_url.clone()) {
            Entry::Occupied(mut v) => {
                let old_feed = v.get();
                if old_feed.get_hash() != new_feed.get_hash() {
                    v.insert(new_feed);
                    true
                } else {
                    false
                }
            }
            Entry::Vacant(v) => {
                v.insert(new_feed);
                true
            }
        }
    }

    pub fn get_feed(&self, source_url: &str) -> Option<&RssFeed> {
        self.rss_feeds.get(source_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE1: &str = "test1";
    const SOURCE2: &str = "test2";
    const SOURCE3: &str = "test3";
    const INVALID_SOURCE: &str = "invalid";

    const FEED1_FILE: &str = "tests/sedaily.rss";
    const FEED2_FILE: &str = "tests/hn.rss";

    fn set_up_scheduler() -> RssScheduler {
        let mut scheduler = RssScheduler::new();
        scheduler.add_source_feed(SOURCE1);
        scheduler.add_source_feed(SOURCE2);
        scheduler.add_source_feed(SOURCE3);

        scheduler
    }

    fn set_up_scheduler_with_feeds() -> RssScheduler {
        let mut scheduler = set_up_scheduler();

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();

        scheduler.add_new_feed(feed1);
        scheduler.add_new_feed(feed2);

        scheduler
    }

    #[test]
    fn add_source_feed() {
        let mut scheduler = RssScheduler::new();

        assert_eq!(scheduler.source_feeds.len(), 0);
        assert_eq!(scheduler.add_source_feed(SOURCE1), true);
        assert_eq!(scheduler.add_source_feed(SOURCE2), true);
        assert_eq!(scheduler.add_source_feed(SOURCE2), false);
        assert_eq!(scheduler.add_source_feed(SOURCE3), true);
        assert_eq!(scheduler.source_feeds.len(), 3);
    }

    #[test]
    fn get_source_feeds() {
        let scheduler = set_up_scheduler();

        assert_eq!(scheduler.get_source_feed(SOURCE1).unwrap().url, SOURCE1);
        assert_eq!(scheduler.get_source_feed(SOURCE2).unwrap().url, SOURCE2);
        assert_eq!(scheduler.get_source_feed(SOURCE3).unwrap().url, SOURCE3);
        assert!(scheduler.get_source_feed(INVALID_SOURCE).is_none());
    }

    #[test]
    fn add_new_feed() {
        let scheduler = set_up_scheduler_with_feeds();

        assert_eq!(scheduler.rss_feeds.len(), 2);
    }

    #[test]
    fn get_new_feed_from_feed_source() {
        let scheduler = set_up_scheduler_with_feeds();

        let feed1 = scheduler.get_feed(SOURCE1).unwrap();
        let feed2 = scheduler.get_feed(SOURCE2).unwrap();
        let feed3 = scheduler.get_feed(INVALID_SOURCE);

        assert_eq!(
            feed1.get_hash(),
            RssFeed::new_from_file(SOURCE1, FEED1_FILE)
                .unwrap()
                .get_hash()
        );
        assert_eq!(
            feed2.get_hash(),
            RssFeed::new_from_file(SOURCE2, FEED2_FILE)
                .unwrap()
                .get_hash()
        );
        assert!(feed3.is_none());
    }

    #[test]
    fn update_feed() {
        let mut scheduler = set_up_scheduler_with_feeds();

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash().to_string();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash().to_string();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash().to_string();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash().to_string();

        assert_eq!(scheduler.get_feed(SOURCE1).unwrap().get_hash(), feed1_hash);
        assert!(!scheduler.add_new_feed(feed1));
        assert_eq!(scheduler.get_feed(SOURCE1).unwrap().get_hash(), feed1_hash);

        assert_eq!(scheduler.get_feed(SOURCE2).unwrap().get_hash(), feed2_hash);
        assert!(!scheduler.add_new_feed(feed2));
        assert_eq!(scheduler.get_feed(SOURCE2).unwrap().get_hash(), feed2_hash);

        assert_ne!(scheduler.get_feed(SOURCE1).unwrap().get_hash(), feed3_hash);
        assert!(scheduler.add_new_feed(feed3));
        assert_ne!(scheduler.get_feed(SOURCE1).unwrap().get_hash(), feed1_hash);
        assert_eq!(scheduler.get_feed(SOURCE1).unwrap().get_hash(), feed3_hash);

        assert_ne!(scheduler.get_feed(SOURCE2).unwrap().get_hash(), feed4_hash);
        assert!(scheduler.add_new_feed(feed4));
        assert_ne!(scheduler.get_feed(SOURCE2).unwrap().get_hash(), feed2_hash);
        assert_eq!(scheduler.get_feed(SOURCE2).unwrap().get_hash(), feed4_hash);
    }
}

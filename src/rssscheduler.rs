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

    pub fn add_new_feed(&mut self, new_feed: RssFeed) {
        let source_url = new_feed.get_source_feed();
        match self.rss_feeds.entry(source_url.clone()) {
            Entry::Occupied(mut v) => {
                let feeds = v.get_mut();
                feeds.push(new_feed);
            }
            Entry::Vacant(v) => {
                v.insert(vec![new_feed]);
            }
        };
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
        let feed3 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();

        scheduler.add_new_feed(feed1);
        scheduler.add_new_feed(feed2);
        scheduler.add_new_feed(feed3);
        scheduler.add_new_feed(feed4);

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
        assert_eq!(scheduler.rss_feeds.get(SOURCE1).unwrap().len(), 2);
        assert_eq!(scheduler.rss_feeds.get(SOURCE2).unwrap().len(), 2);
    }

    #[test]
    fn get_new_feed_from_feed_source() {
        panic!("unimplemented");
    }

    #[test]
    fn check_feed_need_update() {
        panic!("unimplemented");
    }

    #[test]
    fn update_feed() {
        panic!("unimplemented");
    }
}

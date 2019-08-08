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

    pub fn load_source_feeds_from_storage(&mut self, storage: &RssSchedulerStorage) {
        self.source_feeds = storage.get_source_feeds();
    }

    pub fn load_feed_from_storage(&mut self, storage: &RssSchedulerStorage) {
        for source_feed in self.source_feeds.values() {
            let feed = storage.get_last_rss_feed(&source_feed.url);
            if let Some(feed) = feed {
                self.rss_feeds.insert(source_feed.url.clone(), feed);
            }
        }
    }

    pub fn store_feeds_to_database(&self, storage: &mut RssSchedulerStorage) {
        for source_feed in self.source_feeds.values() {
            let feed = self.rss_feeds.get(&source_feed.url);
            if let Some(feed) = feed {
                storage.add_new_rss_feed(feed.clone());
            }
        }
    }

    // TODO: add better return with found errors
    pub fn load_new_feeds_from_source(&mut self) {
        let mut feeds_to_add = vec![];
        for source_feed in self.source_feeds.values() {
            let feed = RssFeed::new_from_url(source_feed.url.as_str());
            if let Ok(feed) = feed {
                feeds_to_add.push(feed);
            }
        }

        for feed in feeds_to_add {
            self.add_new_feed(feed);
        }
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

    const REAL_FEED_URL: &str = "https://softwareengineeringdaily.com/category/podcast/feed";

    struct RssSchedulerStorageTest {
        source_feeds: HashMap<String, SourceFeed>,
        rss_feeds: HashMap<String, Vec<RssFeed>>,
    }

    impl RssSchedulerStorageTest {
        pub fn new() -> RssSchedulerStorageTest {
            RssSchedulerStorageTest {
                source_feeds: HashMap::new(),
                rss_feeds: HashMap::new(),
            }
        }
    }

    impl RssSchedulerStorage for RssSchedulerStorageTest {
        fn get_source_feeds(&self) -> HashMap<String, SourceFeed> {
            self.source_feeds.clone()
        }
        fn get_last_rss_feed(&self, url: &str) -> Option<RssFeed> {
            let feeds = match self.rss_feeds.get(url) {
                None => return None,
                Some(v) => v,
            };
            feeds.last().cloned()
        }
        fn add_new_rss_feed(&mut self, feed: RssFeed) {
            let feeds = self
                .rss_feeds
                .entry(feed.get_source_feed().clone())
                .or_insert(vec![]);
            feeds.push(feed);
        }
        fn add_source_feed(&mut self, source_feed: SourceFeed) {
            self.source_feeds
                .insert(source_feed.url.clone(), source_feed);
        }
    }

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

    #[test]
    fn retrieve_source_feeds_from_database() {
        let mut storage = RssSchedulerStorageTest::new();
        storage.add_source_feed(SourceFeed::new(SOURCE1));
        storage.add_source_feed(SourceFeed::new(SOURCE2));
        storage.add_source_feed(SourceFeed::new(SOURCE3));

        let mut scheduler = RssScheduler::new();
        assert!(scheduler.source_feeds.is_empty());
        scheduler.load_source_feeds_from_storage(&storage);

        assert_eq!(scheduler.source_feeds.len(), 3);

        assert!(scheduler.source_feeds.get(SOURCE1).is_some());
        assert!(scheduler.source_feeds.get(SOURCE2).is_some());
        assert!(scheduler.source_feeds.get(SOURCE3).is_some());
        assert!(scheduler.source_feeds.get(INVALID_SOURCE).is_none());
    }

    #[test]
    fn retrieve_last_rss_feed_from_database() {
        let mut storage = RssSchedulerStorageTest::new();
        storage.add_source_feed(SourceFeed::new(SOURCE1));
        storage.add_source_feed(SourceFeed::new(SOURCE2));
        storage.add_source_feed(SourceFeed::new(SOURCE3));

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash().to_string();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash().to_string();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash().to_string();

        storage.add_new_rss_feed(feed1);
        storage.add_new_rss_feed(feed3);
        storage.add_new_rss_feed(feed2);

        let mut scheduler = set_up_scheduler();
        assert!(scheduler.rss_feeds.is_empty());
        scheduler.load_feed_from_storage(&storage);

        assert_ne!(
            scheduler.rss_feeds.get(SOURCE1).unwrap().get_hash(),
            feed1_hash
        );
        assert_eq!(
            scheduler.rss_feeds.get(SOURCE1).unwrap().get_hash(),
            feed3_hash
        );
        assert_eq!(
            scheduler.rss_feeds.get(SOURCE2).unwrap().get_hash(),
            feed2_hash
        );
    }

    #[test]
    fn add_new_rss_feed_to_database() {
        let mut storage = RssSchedulerStorageTest::new();
        storage.add_source_feed(SourceFeed::new(SOURCE1));
        storage.add_source_feed(SourceFeed::new(SOURCE2));
        storage.add_source_feed(SourceFeed::new(SOURCE3));

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash().to_string();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash().to_string();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash().to_string();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash().to_string();

        let mut scheduler = set_up_scheduler_with_feeds();

        assert!(storage.rss_feeds.get(SOURCE1).is_none());
        assert!(storage.rss_feeds.get(SOURCE2).is_none());
        assert!(storage.rss_feeds.get(SOURCE3).is_none());

        scheduler.store_feeds_to_database(&mut storage);

        assert_eq!(storage.rss_feeds.get(SOURCE1).unwrap().len(), 1);
        assert_eq!(
            storage
                .rss_feeds
                .get(SOURCE1)
                .unwrap()
                .last()
                .unwrap()
                .get_hash(),
            feed1_hash
        );
        assert_eq!(storage.rss_feeds.get(SOURCE2).unwrap().len(), 1);
        assert_eq!(
            storage
                .rss_feeds
                .get(SOURCE2)
                .unwrap()
                .last()
                .unwrap()
                .get_hash(),
            feed2_hash
        );
        assert!(storage.rss_feeds.get(SOURCE3).is_none());

        scheduler.add_new_feed(feed3);
        scheduler.store_feeds_to_database(&mut storage);

        assert_eq!(storage.rss_feeds.get(SOURCE1).unwrap().len(), 2);
        assert_eq!(
            storage
                .rss_feeds
                .get(SOURCE1)
                .unwrap()
                .last()
                .unwrap()
                .get_hash(),
            feed3_hash
        );
        assert_eq!(storage.rss_feeds.get(SOURCE2).unwrap().len(), 2);
        assert_eq!(
            storage
                .rss_feeds
                .get(SOURCE2)
                .unwrap()
                .last()
                .unwrap()
                .get_hash(),
            feed2_hash
        );
        assert!(storage.rss_feeds.get(SOURCE3).is_none());

        scheduler.add_new_feed(feed4);
        scheduler.store_feeds_to_database(&mut storage);

        assert_eq!(storage.rss_feeds.get(SOURCE1).unwrap().len(), 3);
        assert_eq!(
            storage
                .rss_feeds
                .get(SOURCE1)
                .unwrap()
                .last()
                .unwrap()
                .get_hash(),
            feed3_hash
        );
        assert_eq!(storage.rss_feeds.get(SOURCE2).unwrap().len(), 3);
        assert_eq!(
            storage
                .rss_feeds
                .get(SOURCE2)
                .unwrap()
                .last()
                .unwrap()
                .get_hash(),
            feed4_hash
        );
        assert!(storage.rss_feeds.get(SOURCE3).is_none());
    }

    #[test]
    #[ignore]
    fn load_new_feeds_from_url() {
        let mut scheduler = RssScheduler::new();
        scheduler.add_source_feed(REAL_FEED_URL);

        assert!(scheduler.rss_feeds.get(REAL_FEED_URL).is_none());

        scheduler.load_new_feeds_from_source();

        assert!(scheduler.rss_feeds.get(REAL_FEED_URL).is_some());
        assert_eq!(
            scheduler
                .rss_feeds
                .get(REAL_FEED_URL)
                .unwrap()
                .get_source_feed(),
            REAL_FEED_URL
        );
    }

    #[test]
    fn find_new_items_from_new_feeds() {
        panic!("unimplemented");
    }
}

use common::{RssFeed, SourceFeed};

use std::collections::HashMap;
use std::error::Error;

pub trait RssSchedulerStorage {
    fn get_source_feeds(&self) -> Result<HashMap<String, SourceFeed>, Box<dyn Error>>;
    fn get_last_rss_feed(&self, url: &str) -> Result<Option<RssFeed>, Box<dyn Error>>;
    fn add_new_rss_feed(&self, feed: RssFeed) -> Result<(), Box<dyn Error>>;
    fn add_source_feed(&self, source_feed: SourceFeed) -> Result<(), Box<dyn Error>>;
}

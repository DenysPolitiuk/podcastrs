use common::{RssFeed, SourceFeed};

use std::collections::HashMap;
use std::error::Error;

pub trait PodRocketStorage {
    fn get_source_feeds(&self) -> Result<HashMap<String, SourceFeed>, Box<dyn Error>>;
    fn get_rss_feeds(&self) -> Result<HashMap<String, RssFeed>, Box<dyn Error>>;
    fn get_rss_feed_by_id(&self, id: usize) -> Result<Option<RssFeed>, Box<dyn Error>>;
    fn get_rss_feeds_by_url(&self, url: &str) -> Result<Vec<RssFeed>, Box<dyn Error>>;
    fn add_new_rss_feed(&self, feed: RssFeed) -> Result<(), Box<dyn Error>>;
    fn add_source_feed(&self, source_feed: SourceFeed) -> Result<(), Box<dyn Error>>;
}

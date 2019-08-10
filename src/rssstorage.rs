use crate::RssFeed;
use crate::RssSchedulerStorage;
use crate::SourceFeed;

use mongodb::db::ThreadedDatabase;
use mongodb::{bson, doc, Bson};
use mongodb::{Client, ThreadedClient};

use std::collections::HashMap;
use std::error::Error;

static DEFAULT_DATABASE_NAME: &str = "rss";
static DEFAULT_DATABASE_COLLECTION_SOURCE_FEED: &str = "sourcefeed";
static DEFAULT_DATABASE_COLLECTION_RSS_FEED: &str = "rssfeed";

pub struct RssStorageConfig {
    pub database: String,
    pub database_source_feed_collection: String,
    pub database_rss_feed_collection: String,
}

pub struct RssStorage {
    config: RssStorageConfig,
    client: Client,
}

impl RssStorage {
    pub fn new(host: &str, port: u16) -> Result<RssStorage, Box<dyn Error>> {
        let client = Client::connect(host, port)?;
        Ok(RssStorage {
            client,
            config: RssStorageConfig {
                database: DEFAULT_DATABASE_NAME.to_string(),
                database_source_feed_collection: DEFAULT_DATABASE_COLLECTION_SOURCE_FEED
                    .to_string(),
                database_rss_feed_collection: DEFAULT_DATABASE_COLLECTION_RSS_FEED.to_string(),
            },
        })
    }
}

impl RssSchedulerStorage for RssStorage {
    fn get_source_feeds(&self) -> Result<HashMap<String, SourceFeed>, Box<dyn Error>> {
        // let collection = self
        // .client
        // .db(&self.config.database)
        // .collection(&self.config.database_source_feed_collection);

        // // TODO: fix unwrap and put proper error handling
        // let mut cursor = collection.find(None, None).ok().unwrap();

        Ok(HashMap::new())
    }

    fn get_last_rss_feed(&self, url: &str) -> Result<Option<RssFeed>, Box<dyn Error>> {
        Ok(None)
    }

    fn add_new_rss_feed(&self, feed: RssFeed) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    // TODO: fix unwraps
    fn add_source_feed(&self, source_feed: SourceFeed) -> Result<(), Box<dyn Error>> {
        let collection = self
            .client
            .db(&self.config.database)
            .collection(&self.config.database_source_feed_collection);

        let doc = doc! {
            "data": mongodb::to_bson(&source_feed)?,
        };

        collection.insert_one(doc, None)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_HOST: &str = "localhost";
    const DEFAULT_PORT: u16 = 27017;

    static DEFAULT_DATABASE_NAME_TEST: &str = "rss-test";
    static DEFAULT_DATABASE_COLLECTION_SOURCE_FEED_TEST: &str = "sourcefeed-test";
    static DEFAULT_DATABASE_COLLECTION_RSS_FEED_TEST: &str = "rssfeed-test";

    const SOURCE1: &str = "test1";
    const SOURCE2: &str = "test2";
    const SOURCE3: &str = "test3";
    const INVALID_SOURCE: &str = "invalid";

    fn get_test_database_config() -> RssStorageConfig {
        RssStorageConfig {
            database: DEFAULT_DATABASE_NAME_TEST.to_string(),
            database_source_feed_collection: DEFAULT_DATABASE_COLLECTION_SOURCE_FEED_TEST
                .to_string(),
            database_rss_feed_collection: DEFAULT_DATABASE_COLLECTION_RSS_FEED_TEST.to_string(),
        }
    }

    #[test]
    fn add_new_source_feed() {
        let mut storage = RssStorage::new(DEFAULT_HOST, DEFAULT_PORT).unwrap();
        storage.config = get_test_database_config();
        storage.config.database = format!("{},{}", storage.config.database, "add_new_source_feed");

        let source_feed1 = SourceFeed::new(SOURCE1);
        let source_feed2 = SourceFeed::new(SOURCE2);
        let source_feed3 = SourceFeed::new(SOURCE3);

        let mut stored_items = HashMap::new();
        stored_items.insert(source_feed1.url.clone(), source_feed1.clone());
        stored_items.insert(source_feed2.url.clone(), source_feed2.clone());
        stored_items.insert(source_feed3.url.clone(), source_feed3.clone());

        storage.add_source_feed(source_feed1).unwrap();
        storage.add_source_feed(source_feed3).unwrap();
        storage.add_source_feed(source_feed2).unwrap();

        let client = Client::connect(DEFAULT_HOST, DEFAULT_PORT)
            .expect("Failed to initialize standalone client");
        let coll = client
            .db(&storage.config.database)
            .collection(&storage.config.database_source_feed_collection);

        let mut cursor = coll.find(None, None).ok().expect("Failed to execute find");

        let mut result = vec![];

        for item in cursor {
            let doc = match item {
                Ok(doc) => doc,
                Err(e) => panic!(format!("Failed to get next from server! {}", e)),
            };
            result.push(doc);
        }

        for doc in result {
            let son = doc.get("data").unwrap();
            let source_feed: SourceFeed = bson::from_bson(son.clone()).unwrap();

            stored_items.remove(&source_feed.url);
        }

        assert!(stored_items.is_empty());

        coll.drop().unwrap();
    }
}

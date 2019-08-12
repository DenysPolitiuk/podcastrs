use common::RssFeed;
use common::SourceFeed;
use scheduler_trait::RssSchedulerStorage;

use mongodb::db::ThreadedDatabase;
use mongodb::{bson, doc};
use mongodb::{Client, ThreadedClient};
use serde_json;

use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

static DEFAULT_DATABASE_NAME: &str = "rss";
static DEFAULT_DATABASE_COLLECTION_SOURCE_FEED: &str = "sourcefeed";
static DEFAULT_DATABASE_COLLECTION_RSS_FEED: &str = "rssfeed";

static DATA_FIELD: &str = "data";
static TIMESTAMP_FIELD: &str = "timestamp";
static SOURCE_URL_FIELD: &str = "url";

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
        let collection = self
            .client
            .db(&self.config.database)
            .collection(&self.config.database_source_feed_collection);

        let cursor = collection.find(None, None)?;

        let mut results = HashMap::new();

        // TODO: better handling of bad data
        for item in cursor {
            let doc = match item {
                Ok(doc) => doc,
                Err(e) => Err(format!("Failed to get next from server! {}", e))?,
            };
            let son = match doc.get(DATA_FIELD) {
                Some(v) => v,
                None => Err(format!("No `{}` field in item", DATA_FIELD))?,
            };
            let source_feed: SourceFeed = match bson::from_bson(son.clone()) {
                Ok(v) => v,
                Err(e) => Err(format!("Failed to parse input BSON, error : {}", e))?,
            };

            results.insert(source_feed.url.clone(), source_feed);
        }

        Ok(results)
    }

    fn get_last_rss_feed(&self, url: &str) -> Result<Option<RssFeed>, Box<dyn Error>> {
        let collection = self
            .client
            .db(&self.config.database)
            .collection(&self.config.database_rss_feed_collection);

        let find_doc = doc! {
            SOURCE_URL_FIELD: url,
        };
        let find_sort_doc = doc! {
            TIMESTAMP_FIELD: -1,
        };
        let mut find_options = mongodb::coll::options::FindOptions::new();
        find_options.limit = Some(1);
        find_options.sort = Some(find_sort_doc);

        let result = collection.find_one(Some(find_doc), Some(find_options))?;

        let doc = match result {
            None => return Ok(None),
            Some(v) => v,
        };

        let son = doc.get(DATA_FIELD).unwrap();
        let feed_json: String = bson::from_bson(son.clone())?;
        let feed: RssFeed = serde_json::from_str(&feed_json)?;
        let result_feed = Some(feed);

        Ok(result_feed)
    }

    fn add_new_rss_feed(&self, feed: RssFeed) -> Result<(), Box<dyn Error>> {
        let collection = self
            .client
            .db(&self.config.database)
            .collection(&self.config.database_rss_feed_collection);

        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)?;

        let doc = doc! {
            TIMESTAMP_FIELD: format!("{}", since_the_epoch.as_millis()),
            SOURCE_URL_FIELD: feed.get_source_feed(),
            DATA_FIELD: mongodb::to_bson(&serde_json::to_string(&feed)?)?,
        };

        collection.insert_one(doc, None)?;

        Ok(())
    }

    // TODO: fix unwraps
    fn add_source_feed(&self, source_feed: SourceFeed) -> Result<(), Box<dyn Error>> {
        let collection = self
            .client
            .db(&self.config.database)
            .collection(&self.config.database_source_feed_collection);

        let doc = doc! {
            DATA_FIELD: mongodb::to_bson(&source_feed)?,
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

    const FEED1_FILE: &str = "../tests/sedaily.rss";
    const FEED2_FILE: &str = "../tests/hn.rss";

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
        storage.config.database_source_feed_collection = format!(
            "{},{}",
            storage.config.database_source_feed_collection, "add_new_source_feed"
        );

        let client = Client::connect(DEFAULT_HOST, DEFAULT_PORT)
            .expect("Failed to initialize standalone client");
        let coll = client
            .db(&storage.config.database)
            .collection(&storage.config.database_source_feed_collection);

        coll.drop().unwrap();

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

        let cursor = coll.find(None, None).ok().expect("Failed to execute find");

        let mut result = vec![];

        for item in cursor {
            let doc = match item {
                Ok(doc) => doc,
                Err(e) => panic!(format!("Failed to get next from server! {}", e)),
            };
            result.push(doc);
        }

        for doc in result {
            let son = doc.get(DATA_FIELD).unwrap();
            let source_feed: SourceFeed = bson::from_bson(son.clone()).unwrap();

            stored_items.remove(&source_feed.url);
        }

        assert!(stored_items.is_empty());

        coll.drop().unwrap();
    }

    #[test]
    fn get_source_feeds() {
        let mut storage = RssStorage::new(DEFAULT_HOST, DEFAULT_PORT).unwrap();
        storage.config = get_test_database_config();
        storage.config.database_source_feed_collection = format!(
            "{},{}",
            storage.config.database_source_feed_collection, "get_source_feeds"
        );

        let client = Client::connect(DEFAULT_HOST, DEFAULT_PORT)
            .expect("Failed to initialize standalone client");
        let coll = client
            .db(&storage.config.database)
            .collection(&storage.config.database_source_feed_collection);

        coll.drop().unwrap();

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

        let results = storage.get_source_feeds().unwrap();

        assert_eq!(results.len(), stored_items.len());
        for item in results.keys() {
            assert_eq!(
                results.get(&item.clone()).unwrap().url,
                stored_items.get(&item.clone()).unwrap().url
            );
        }

        coll.drop().unwrap();
    }

    #[test]
    fn add_new_rss_feed() {
        let mut storage = RssStorage::new(DEFAULT_HOST, DEFAULT_PORT).unwrap();
        storage.config = get_test_database_config();
        storage.config.database_rss_feed_collection = format!(
            "{},{}",
            storage.config.database_rss_feed_collection, "add_new_rss_feed"
        );

        let client = Client::connect(DEFAULT_HOST, DEFAULT_PORT)
            .expect("Failed to initialize standalone client");
        let coll = client
            .db(&storage.config.database)
            .collection(&storage.config.database_rss_feed_collection);

        coll.drop().unwrap();

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash().to_string();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash().to_string();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash().to_string();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash().to_string();

        let mut stored_items = HashMap::new();
        stored_items.insert(feed1_hash.clone(), feed1.clone());
        stored_items.insert(feed2_hash.clone(), feed2.clone());
        stored_items.insert(feed3_hash.clone(), feed3.clone());
        stored_items.insert(feed4_hash.clone(), feed4.clone());

        storage.add_new_rss_feed(feed1).unwrap();
        storage.add_new_rss_feed(feed2).unwrap();
        storage.add_new_rss_feed(feed3).unwrap();
        storage.add_new_rss_feed(feed4).unwrap();

        let cursor = coll.find(None, None).ok().expect("Failed to execute find");

        let mut result = vec![];

        for item in cursor {
            let doc = match item {
                Ok(doc) => doc,
                Err(e) => panic!(format!("Failed to get next from server! {}", e)),
            };
            result.push(doc);
        }

        for doc in result {
            let son = doc.get(DATA_FIELD).unwrap();
            let feed_json: String = bson::from_bson(son.clone()).unwrap();
            let feed: RssFeed = serde_json::from_str(&feed_json).unwrap();

            stored_items.remove(feed.get_hash());
        }

        assert!(stored_items.is_empty());

        coll.drop().unwrap();
    }

    #[test]
    fn get_last_rss_feed() {
        let mut storage = RssStorage::new(DEFAULT_HOST, DEFAULT_PORT).unwrap();
        storage.config = get_test_database_config();
        storage.config.database_rss_feed_collection = format!(
            "{},{}",
            storage.config.database_rss_feed_collection, "get_last_rss_feed"
        );

        let client = Client::connect(DEFAULT_HOST, DEFAULT_PORT)
            .expect("Failed to initialize standalone client");
        let coll = client
            .db(&storage.config.database)
            .collection(&storage.config.database_rss_feed_collection);

        coll.drop().unwrap();

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash().to_string();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash().to_string();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash().to_string();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash().to_string();

        assert!(storage.get_last_rss_feed(SOURCE1).unwrap().is_none());

        storage.add_new_rss_feed(feed1.clone()).unwrap();
        let feed = storage.get_last_rss_feed(SOURCE1).unwrap().unwrap();
        assert_eq!(feed1_hash, feed.get_hash());

        storage.add_new_rss_feed(feed4.clone()).unwrap();
        let feed = storage.get_last_rss_feed(SOURCE2).unwrap().unwrap();
        assert_eq!(feed4_hash, feed.get_hash());

        storage.add_new_rss_feed(feed3.clone()).unwrap();
        let feed = storage.get_last_rss_feed(SOURCE1).unwrap().unwrap();
        assert_eq!(feed3_hash, feed.get_hash());

        storage.add_new_rss_feed(feed2.clone()).unwrap();
        let feed = storage.get_last_rss_feed(SOURCE2).unwrap().unwrap();
        assert_eq!(feed2_hash, feed.get_hash());

        coll.drop().unwrap();
    }
}

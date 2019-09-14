use common::RssFeed;
use common::SourceFeed;
use podrocket_trait::PodRocketStorage;
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

#[derive(Clone)]
pub struct RssStorageConfig {
    database: String,
    database_source_feed_collection: String,
    database_rss_feed_collection: String,
}

impl RssStorageConfig {
    pub fn new() -> RssStorageConfig {
        RssStorageConfig {
            database: DEFAULT_DATABASE_NAME.to_string(),
            database_source_feed_collection: DEFAULT_DATABASE_COLLECTION_SOURCE_FEED.to_string(),
            database_rss_feed_collection: DEFAULT_DATABASE_COLLECTION_RSS_FEED.to_string(),
        }
    }

    pub fn with_database_name(mut self, database_name: &str) -> Self {
        self.database = database_name.to_string();
        self
    }

    pub fn with_source_feed_collection_name(mut self, source_feed_collection: &str) -> Self {
        self.database_source_feed_collection = source_feed_collection.to_string();
        self
    }

    pub fn with_rss_feed_collection_name(mut self, rss_feed_collection: &str) -> Self {
        self.database_rss_feed_collection = rss_feed_collection.to_string();
        self
    }
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

    pub fn with_config(mut self, config: &RssStorageConfig) -> Self {
        self.config = config.clone();
        self
    }

    pub fn drop_collection(&self, collection_name: &str) -> Result<(), Box<dyn Error>> {
        self.client
            .db(&self.config.database)
            .collection(collection_name)
            .drop()?;

        Ok(())
    }

    // TODO: fix unwraps
    pub fn add_source_feed(&self, source_feed: SourceFeed) -> Result<(), Box<dyn Error>> {
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

    pub fn add_new_rss_feed(&self, feed: RssFeed) -> Result<(), Box<dyn Error>> {
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

    pub fn get_source_feeds(&self) -> Result<HashMap<String, SourceFeed>, Box<dyn Error>> {
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

            results.insert(source_feed.get_url(), source_feed);
        }

        Ok(results)
    }
}

impl RssSchedulerStorage for RssStorage {
    fn get_source_feeds(&self) -> Result<HashMap<String, SourceFeed>, Box<dyn Error>> {
        self.get_source_feeds()
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
        self.add_new_rss_feed(feed)
    }

    fn add_source_feed(&self, source_feed: SourceFeed) -> Result<(), Box<dyn Error>> {
        self.add_source_feed(source_feed)
    }
}

impl PodRocketStorage for RssStorage {
    fn get_source_feeds(&self) -> Result<HashMap<String, SourceFeed>, Box<dyn Error>> {
        self.get_source_feeds()
    }

    fn get_source_feed_by_url(&self, url: &str) -> Result<Option<SourceFeed>, Box<dyn Error>> {
        let collection = self
            .client
            .db(&self.config.database)
            .collection(&self.config.database_source_feed_collection);

        let find_doc = doc! {
            format!("{}.{}", DATA_FIELD, "url"): url
        };
        let mut find_options = mongodb::coll::options::FindOptions::new();
        find_options.limit = Some(1);

        let result = collection.find_one(Some(find_doc), Some(find_options))?;

        let doc = match result {
            None => return Ok(None),
            Some(v) => v,
        };

        let son = match doc.get(DATA_FIELD) {
            Some(v) => v,
            None => Err(format!("No `{}` field in item", DATA_FIELD))?,
        };
        let source_feed: SourceFeed = match bson::from_bson(son.clone()) {
            Ok(v) => v,
            Err(e) => Err(format!("Failed to parse input BSON, error : {}", e))?,
        };

        Ok(Some(source_feed))
    }

    fn get_rss_feeds(&self) -> Result<HashMap<String, Vec<RssFeed>>, Box<dyn Error>> {
        let collection = self
            .client
            .db(&self.config.database)
            .collection(&self.config.database_rss_feed_collection);

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
            let feed_json: String = bson::from_bson(son.clone())?;
            let feed: RssFeed = serde_json::from_str(&feed_json)?;

            results
                .entry(feed.get_source_feed().clone())
                .or_insert(Vec::new())
                .push(feed);
        }

        Ok(results)
    }

    fn get_rss_feeds_latest(&self) -> Result<HashMap<String, RssFeed>, Box<dyn Error>> {
        let source_feeds = self.get_source_feeds()?;
        if source_feeds.is_empty() {
            return Ok(HashMap::new());
        }

        let collection = self
            .client
            .db(&self.config.database)
            .collection(&self.config.database_rss_feed_collection);

        let mut result_feeds = HashMap::new();

        for source_feed in source_feeds.values() {
            let find_doc = doc! {
                SOURCE_URL_FIELD: source_feed.get_url(),
            };
            let find_sort_doc = doc! {
                TIMESTAMP_FIELD: -1,
            };

            let mut find_options = mongodb::coll::options::FindOptions::new();
            find_options.limit = Some(1);
            find_options.sort = Some(find_sort_doc);

            let result = collection.find_one(Some(find_doc), Some(find_options))?;

            let doc = match result {
                None => continue,
                Some(v) => v,
            };

            let son = doc.get(DATA_FIELD).unwrap();
            let feed_json: String = bson::from_bson(son.clone())?;
            let feed: RssFeed = serde_json::from_str(&feed_json)?;
            result_feeds.insert(source_feed.get_url(), feed);
        }

        Ok(result_feeds)
    }

    fn get_rss_feed_by_id(&self, id: &str) -> Result<Option<RssFeed>, Box<dyn Error>> {
        let collection = self
            .client
            .db(&self.config.database)
            .collection(&self.config.database_rss_feed_collection);

        let find_doc = doc! {
            "_id": mongodb::oid::ObjectId::with_string(id)?,
        };
        let mut find_options = mongodb::coll::options::FindOptions::new();
        find_options.limit = Some(1);

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

    fn get_rss_feeds_by_url(&self, url: &str) -> Result<Vec<RssFeed>, Box<dyn Error>> {
        let collection = self
            .client
            .db(&self.config.database)
            .collection(&self.config.database_rss_feed_collection);

        let find_doc = doc! {
            SOURCE_URL_FIELD: url,
        };
        let cursor = collection.find(Some(find_doc), None)?;

        let mut results = Vec::new();

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
            let feed_json: String = bson::from_bson(son.clone())?;
            let feed: RssFeed = serde_json::from_str(&feed_json)?;

            results.push(feed);
        }

        Ok(results)
    }

    fn add_new_rss_feed(&self, feed: RssFeed) -> Result<(), Box<dyn Error>> {
        self.add_new_rss_feed(feed)
    }

    fn add_source_feed(&self, source_feed: SourceFeed) -> Result<(), Box<dyn Error>> {
        self.add_source_feed(source_feed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use mongodb::coll::Collection;

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

        let source_feed1 = SourceFeed::new(SOURCE1, "").unwrap();
        let source_feed2 = SourceFeed::new(SOURCE2, "").unwrap();
        let source_feed3 = SourceFeed::new(SOURCE3, "").unwrap();

        let mut stored_items = HashMap::new();
        stored_items.insert(source_feed1.get_url(), source_feed1.clone());
        stored_items.insert(source_feed2.get_url(), source_feed2.clone());
        stored_items.insert(source_feed3.get_url(), source_feed3.clone());

        RssStorage::add_source_feed(&storage, source_feed1).unwrap();
        RssStorage::add_source_feed(&storage, source_feed3).unwrap();
        RssStorage::add_source_feed(&storage, source_feed2).unwrap();

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

            stored_items.remove(&source_feed.get_url());
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

        let source_feed1 = SourceFeed::new(SOURCE1, "").unwrap();
        let source_feed2 = SourceFeed::new(SOURCE2, "").unwrap();
        let source_feed3 = SourceFeed::new(SOURCE3, "").unwrap();

        let mut stored_items = HashMap::new();
        stored_items.insert(source_feed1.get_url(), source_feed1.clone());
        stored_items.insert(source_feed2.get_url(), source_feed2.clone());
        stored_items.insert(source_feed3.get_url(), source_feed3.clone());

        RssStorage::add_source_feed(&storage, source_feed1).unwrap();
        RssStorage::add_source_feed(&storage, source_feed3).unwrap();
        RssStorage::add_source_feed(&storage, source_feed2).unwrap();

        let results = RssStorage::get_source_feeds(&storage).unwrap();

        assert_eq!(results.len(), stored_items.len());
        for item in results.keys() {
            assert_eq!(
                results.get(&item.clone()).unwrap().get_url(),
                stored_items.get(&item.clone()).unwrap().get_url()
            );
        }

        coll.drop().unwrap();
    }

    #[test]
    fn get_source_feed_by_url() {
        let mut storage = RssStorage::new(DEFAULT_HOST, DEFAULT_PORT).unwrap();
        storage.config = get_test_database_config();
        storage.config.database_source_feed_collection = format!(
            "{},{}",
            storage.config.database_source_feed_collection, "get_source_feed_by_url"
        );

        let client = Client::connect(DEFAULT_HOST, DEFAULT_PORT)
            .expect("Failed to initialize standalone client");
        let coll = client
            .db(&storage.config.database)
            .collection(&storage.config.database_source_feed_collection);

        coll.drop().unwrap();

        let source_feed1 = SourceFeed::new(SOURCE1, "").unwrap();
        let source_feed2 = SourceFeed::new(SOURCE2, "").unwrap();
        let source_feed3 = SourceFeed::new(SOURCE3, "").unwrap();

        let mut stored_items = HashMap::new();
        stored_items.insert(source_feed1.get_url(), source_feed1.clone());
        stored_items.insert(source_feed2.get_url(), source_feed2.clone());
        stored_items.insert(source_feed3.get_url(), source_feed3.clone());

        assert!(RssStorage::get_source_feed_by_url(&storage, SOURCE1)
            .unwrap()
            .is_none());

        RssStorage::add_source_feed(&storage, source_feed1.clone()).unwrap();
        RssStorage::add_source_feed(&storage, source_feed3.clone()).unwrap();
        RssStorage::add_source_feed(&storage, source_feed2.clone()).unwrap();

        let result = RssStorage::get_source_feed_by_url(&storage, SOURCE1)
            .unwrap()
            .unwrap();
        assert_eq!(result.get_url(), source_feed1.get_url());

        let result = RssStorage::get_source_feed_by_url(&storage, SOURCE2)
            .unwrap()
            .unwrap();
        assert_eq!(result.get_url(), source_feed2.get_url());

        let result = RssStorage::get_source_feed_by_url(&storage, SOURCE3)
            .unwrap()
            .unwrap();
        assert_eq!(result.get_url(), source_feed3.get_url());

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
        let feed1_hash = feed1.get_hash();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash();

        let mut stored_items = HashMap::new();
        stored_items.insert(feed1_hash, feed1.clone());
        stored_items.insert(feed2_hash, feed2.clone());
        stored_items.insert(feed3_hash, feed3.clone());
        stored_items.insert(feed4_hash, feed4.clone());

        RssStorage::add_new_rss_feed(&storage, feed1).unwrap();
        RssStorage::add_new_rss_feed(&storage, feed2).unwrap();
        RssStorage::add_new_rss_feed(&storage, feed3).unwrap();
        RssStorage::add_new_rss_feed(&storage, feed4).unwrap();

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

            stored_items.remove(&feed.get_hash());
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
        let feed1_hash = feed1.get_hash();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash();

        assert!(storage.get_last_rss_feed(SOURCE1).unwrap().is_none());

        RssStorage::add_new_rss_feed(&storage, feed1.clone()).unwrap();
        let feed = storage.get_last_rss_feed(SOURCE1).unwrap().unwrap();
        assert_eq!(feed1_hash, feed.get_hash());

        RssStorage::add_new_rss_feed(&storage, feed4.clone()).unwrap();
        let feed = storage.get_last_rss_feed(SOURCE2).unwrap().unwrap();
        assert_eq!(feed4_hash, feed.get_hash());

        RssStorage::add_new_rss_feed(&storage, feed3.clone()).unwrap();
        let feed = storage.get_last_rss_feed(SOURCE1).unwrap().unwrap();
        assert_eq!(feed3_hash, feed.get_hash());

        RssStorage::add_new_rss_feed(&storage, feed2.clone()).unwrap();
        let feed = storage.get_last_rss_feed(SOURCE2).unwrap().unwrap();
        assert_eq!(feed2_hash, feed.get_hash());

        coll.drop().unwrap();
    }

    fn get_rss_feed_id(coll: &Collection, feed: &RssFeed) -> String {
        let mut find_options = mongodb::coll::options::FindOptions::new();
        find_options.limit = Some(1);

        let find_doc = doc! {
            DATA_FIELD: mongodb::to_bson(&serde_json::to_string(&feed).unwrap()).unwrap(),
        };
        let doc = coll
            .find_one(Some(find_doc), Some(find_options))
            .unwrap()
            .unwrap();
        format!("{}", doc.get_object_id("_id").unwrap())
    }

    #[test]
    fn verify_get_rss_feed_by_id() {
        let mut storage = RssStorage::new(DEFAULT_HOST, DEFAULT_PORT).unwrap();
        storage.config = get_test_database_config();
        storage.config.database_rss_feed_collection = format!(
            "{},{}",
            storage.config.database_rss_feed_collection, "verify_get_rss_feed_by_id"
        );

        let client = Client::connect(DEFAULT_HOST, DEFAULT_PORT)
            .expect("Failed to initialize standalone client");
        let coll = client
            .db(&storage.config.database)
            .collection(&storage.config.database_rss_feed_collection);

        coll.drop().unwrap();

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash();

        RssStorage::add_new_rss_feed(&storage, feed1.clone()).unwrap();
        let id = get_rss_feed_id(&coll, &feed1);
        let _ = storage.get_rss_feeds();
        let feed = storage.get_rss_feed_by_id(id.as_str()).unwrap().unwrap();
        assert_eq!(feed.get_hash(), feed1_hash);

        RssStorage::add_new_rss_feed(&storage, feed4.clone()).unwrap();
        let id = get_rss_feed_id(&coll, &feed4);
        let feed = storage.get_rss_feed_by_id(id.as_str()).unwrap().unwrap();
        assert_eq!(feed.get_hash(), feed4_hash);

        RssStorage::add_new_rss_feed(&storage, feed3.clone()).unwrap();
        let id = get_rss_feed_id(&coll, &feed3);
        let feed = storage.get_rss_feed_by_id(id.as_str()).unwrap().unwrap();
        assert_eq!(feed.get_hash(), feed3_hash);

        RssStorage::add_new_rss_feed(&storage, feed2.clone()).unwrap();
        let id = get_rss_feed_id(&coll, &feed2);
        let feed = storage.get_rss_feed_by_id(id.as_str()).unwrap().unwrap();
        assert_eq!(feed.get_hash(), feed2_hash);

        coll.drop().unwrap();
    }

    #[test]
    fn verify_get_rss_feed_by_url() {
        let mut storage = RssStorage::new(DEFAULT_HOST, DEFAULT_PORT).unwrap();
        storage.config = get_test_database_config();
        storage.config.database_rss_feed_collection = format!(
            "{},{}",
            storage.config.database_rss_feed_collection, "verify_get_rss_feed_by_url"
        );

        let client = Client::connect(DEFAULT_HOST, DEFAULT_PORT)
            .expect("Failed to initialize standalone client");
        let coll = client
            .db(&storage.config.database)
            .collection(&storage.config.database_rss_feed_collection);

        coll.drop().unwrap();

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash();

        RssStorage::add_new_rss_feed(&storage, feed1.clone()).unwrap();
        let feeds = storage.get_rss_feeds_by_url(SOURCE1).unwrap();
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].get_hash(), feed1_hash);

        RssStorage::add_new_rss_feed(&storage, feed4.clone()).unwrap();
        let feeds = storage.get_rss_feeds_by_url(SOURCE2).unwrap();
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds[0].get_hash(), feed4_hash);

        RssStorage::add_new_rss_feed(&storage, feed3.clone()).unwrap();
        let feeds = storage.get_rss_feeds_by_url(SOURCE1).unwrap();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds[1].get_hash(), feed3_hash);

        RssStorage::add_new_rss_feed(&storage, feed2.clone()).unwrap();
        let feeds = storage.get_rss_feeds_by_url(SOURCE2).unwrap();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds[1].get_hash(), feed2_hash);

        coll.drop().unwrap();
    }

    #[test]
    fn verify_get_rss_feeds() {
        let mut storage = RssStorage::new(DEFAULT_HOST, DEFAULT_PORT).unwrap();
        storage.config = get_test_database_config();
        storage.config.database_rss_feed_collection = format!(
            "{},{}",
            storage.config.database_rss_feed_collection, "verify_get_rss_feeds"
        );

        let client = Client::connect(DEFAULT_HOST, DEFAULT_PORT)
            .expect("Failed to initialize standalone client");
        let coll = client
            .db(&storage.config.database)
            .collection(&storage.config.database_rss_feed_collection);

        coll.drop().unwrap();

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash();

        assert!(storage.get_rss_feeds().unwrap().is_empty());

        RssStorage::add_new_rss_feed(&storage, feed1.clone()).unwrap();
        let feeds = storage.get_rss_feeds().unwrap();
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds.get(SOURCE1).unwrap().len(), 1);
        assert_eq!(feeds.get(SOURCE1).unwrap()[0].get_hash(), feed1_hash);

        RssStorage::add_new_rss_feed(&storage, feed4.clone()).unwrap();
        let feeds = storage.get_rss_feeds().unwrap();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds.get(SOURCE2).unwrap().len(), 1);
        assert_eq!(feeds.get(SOURCE2).unwrap()[0].get_hash(), feed4_hash);

        RssStorage::add_new_rss_feed(&storage, feed3.clone()).unwrap();
        let feeds = storage.get_rss_feeds().unwrap();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds.get(SOURCE1).unwrap().len(), 2);
        assert_eq!(feeds.get(SOURCE1).unwrap()[1].get_hash(), feed3_hash);

        RssStorage::add_new_rss_feed(&storage, feed2.clone()).unwrap();
        let feeds = storage.get_rss_feeds().unwrap();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds.get(SOURCE2).unwrap().len(), 2);
        assert_eq!(feeds.get(SOURCE2).unwrap()[1].get_hash(), feed2_hash);

        coll.drop().unwrap();
    }

    #[test]
    fn verify_get_rss_feeds_latest() {
        let mut storage = RssStorage::new(DEFAULT_HOST, DEFAULT_PORT).unwrap();
        storage.config = get_test_database_config();
        storage.config.database_rss_feed_collection = format!(
            "{},{}",
            storage.config.database_rss_feed_collection, "verify_get_rss_feeds_latest"
        );

        let client = Client::connect(DEFAULT_HOST, DEFAULT_PORT)
            .expect("Failed to initialize standalone client");
        let coll = client
            .db(&storage.config.database)
            .collection(&storage.config.database_rss_feed_collection);

        coll.drop().unwrap();

        let source_feed1 = SourceFeed::new(SOURCE1, "").unwrap();
        let source_feed2 = SourceFeed::new(SOURCE2, "").unwrap();
        let source_feed3 = SourceFeed::new(SOURCE3, "").unwrap();

        RssStorage::add_source_feed(&storage, source_feed1).unwrap();
        RssStorage::add_source_feed(&storage, source_feed2).unwrap();
        RssStorage::add_source_feed(&storage, source_feed3).unwrap();

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash();

        assert!(storage.get_rss_feeds().unwrap().is_empty());

        RssStorage::add_new_rss_feed(&storage, feed1.clone()).unwrap();
        let feeds = storage.get_rss_feeds_latest().unwrap();
        assert_eq!(feeds.len(), 1);
        assert_eq!(feeds.get(SOURCE1).unwrap().get_hash(), feed1_hash);

        RssStorage::add_new_rss_feed(&storage, feed4.clone()).unwrap();
        let feeds = storage.get_rss_feeds_latest().unwrap();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds.get(SOURCE1).unwrap().get_hash(), feed1_hash);
        assert_eq!(feeds.get(SOURCE2).unwrap().get_hash(), feed4_hash);

        RssStorage::add_new_rss_feed(&storage, feed3.clone()).unwrap();
        let feeds = storage.get_rss_feeds_latest().unwrap();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds.get(SOURCE1).unwrap().get_hash(), feed3_hash);
        assert_eq!(feeds.get(SOURCE2).unwrap().get_hash(), feed4_hash);

        RssStorage::add_new_rss_feed(&storage, feed2.clone()).unwrap();
        let feeds = storage.get_rss_feeds_latest().unwrap();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds.get(SOURCE1).unwrap().get_hash(), feed3_hash);
        assert_eq!(feeds.get(SOURCE2).unwrap().get_hash(), feed2_hash);

        coll.drop().unwrap();
    }
}

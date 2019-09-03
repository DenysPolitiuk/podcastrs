#![feature(proc_macro_hygiene, decl_macro)]
use rocket::http::uri::Uri;
use rocket::Rocket;
use rocket::State;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;

use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::Arc;

use common::RssFeed;
use common::SourceFeed;
use podrocket_trait::PodRocketStorage;
use storage::RssStorage;

static DEFAULT_HOST: &str = "localhost";
static DEFAULT_PORT: u16 = 27017;

static HOST_ENV_VAR: &str = "RSS_DATABASE_HOST";
static PORT_ENV_VAR: &str = "RSS_DATABASE_PORT";

type Storage = dyn PodRocketStorage + Send + Sync;

#[get("/ping")]
fn ping_pong() -> &'static str {
    "pong"
}

#[get("/source")]
fn get_all_source_feeds(
    storage: State<Arc<Storage>>,
) -> Result<Option<Json<Vec<SourceFeed>>>, Box<dyn Error>> {
    let source_feeds = storage.get_source_feeds()?;

    let source_feeds_vec = source_feeds
        .values()
        .into_iter()
        .map(|sf| sf.clone())
        .collect();

    Ok(Some(Json(source_feeds_vec)))
}

#[get("/source/<id>")]
fn get_source_feed(
    storage: State<Arc<Storage>>,
    id: String,
) -> Result<Option<Json<SourceFeed>>, Box<dyn Error>> {
    Ok(storage
        .get_source_feed_by_url(id.as_str())?
        .map(|f| Json(f)))
}

#[post("/source", format = "application/json", data = "<data>")]
fn post_source_feed(
    storage: State<Arc<Storage>>,
    data: Json<SourceFeed>,
) -> Result<(), Box<dyn Error>> {
    storage.add_source_feed(data.into_inner())
}

#[get("/rss")]
fn get_all_rss(
    storage: State<Arc<Storage>>,
) -> Result<Json<HashMap<String, Vec<RssFeed>>>, Box<dyn Error>> {
    Ok(Json(storage.get_rss_feeds()?))
}

#[get("/rss/<id>")]
fn get_rss_feeds_by_id(
    storage: State<Arc<Storage>>,
    id: String,
) -> Result<Option<Json<RssFeed>>, Box<dyn Error>> {
    Ok(storage.get_rss_feed_by_id(id.as_str())?.map(|f| Json(f)))
}

#[get("/rss/url/<url>")]
fn get_rss_feeds_by_url(
    storage: State<Arc<Storage>>,
    url: String,
) -> Result<Json<Vec<RssFeed>>, Box<dyn Error>> {
    let decoded_url = Uri::percent_decode(url.as_bytes())?;
    Ok(Json(
        storage.get_rss_feeds_by_url(decoded_url.to_string().as_str())?,
    ))
}

fn make_rocket(database_client: Arc<Storage>) -> Rocket {
    rocket::ignite().manage(database_client).mount(
        "/",
        routes![
            ping_pong,
            get_all_source_feeds,
            get_source_feed,
            post_source_feed,
            get_all_rss,
            get_rss_feeds_by_id,
            get_rss_feeds_by_url,
        ],
    )
}

fn main() {
    let storage = RssStorage::new(
        env::var(HOST_ENV_VAR)
            .ok()
            .or(Some(DEFAULT_HOST.to_string()))
            .unwrap()
            .as_str(),
        env::var(PORT_ENV_VAR)
            .ok()
            .or(Some(DEFAULT_PORT.to_string()))
            .unwrap()
            .parse::<u16>()
            .expect("invalid port provided in env vars"),
    )
    .expect("unable to create storate");
    make_rocket(Arc::new(storage)).launch();
}

#[cfg(test)]
mod tests {
    use super::*;

    use rocket::http::{ContentType, Status};
    use rocket::local::Client;
    use serde_json;

    use common::{RssFeed, SourceFeed};

    use std::collections::HashMap;
    use std::error::Error;
    use std::sync::Mutex;

    static SOURCE_FEED_URL: &str = "/source";
    static RSS_FEED_URL: &str = "/rss";

    const SOURCE1: &str = "test1";
    const SOURCE2: &str = "test2";
    const SOURCE3: &str = "test3";

    const FEED1_FILE: &str = "../tests/sedaily.rss";
    const FEED2_FILE: &str = "../tests/hn.rss";

    struct PodRocketStorageTest {
        source_feeds: Mutex<HashMap<String, SourceFeed>>,
        rss_feeds: Mutex<HashMap<String, Vec<RssFeed>>>,
    }

    impl PodRocketStorageTest {
        pub fn new() -> PodRocketStorageTest {
            PodRocketStorageTest {
                source_feeds: Mutex::new(HashMap::new()),
                rss_feeds: Mutex::new(HashMap::new()),
            }
        }
    }

    impl PodRocketStorage for PodRocketStorageTest {
        fn get_rss_feeds(&self) -> Result<HashMap<String, Vec<RssFeed>>, Box<dyn Error>> {
            Ok(self.rss_feeds.lock().unwrap().clone())
        }

        fn get_rss_feed_by_id(&self, id: &str) -> Result<Option<RssFeed>, Box<dyn Error>> {
            for feeds in self.rss_feeds.lock().unwrap().values() {
                for feed in feeds {
                    if feed.get_hash() == id {
                        return Ok(Some(feed.clone()));
                    }
                }
            }

            Ok(None)
        }

        fn get_rss_feeds_by_url(&self, url: &str) -> Result<Vec<RssFeed>, Box<dyn Error>> {
            Ok(match self.rss_feeds.lock().unwrap().get(url) {
                Some(v) => v.clone(),
                None => vec![],
            })
        }

        fn get_source_feeds(&self) -> Result<HashMap<String, SourceFeed>, Box<dyn Error>> {
            Ok(self.source_feeds.lock().unwrap().clone())
        }

        fn get_source_feed_by_url(&self, url: &str) -> Result<Option<SourceFeed>, Box<dyn Error>> {
            Ok(self.source_feeds.lock().unwrap().get(url).cloned())
        }

        fn add_new_rss_feed(&self, feed: RssFeed) -> Result<(), Box<dyn Error>> {
            self.rss_feeds
                .lock()
                .unwrap()
                .entry(feed.get_source_feed().clone())
                .or_insert(vec![])
                .push(feed);

            Ok(())
        }

        fn add_source_feed(&self, source_feed: SourceFeed) -> Result<(), Box<dyn Error>> {
            self.source_feeds
                .lock()
                .unwrap()
                .insert(source_feed.url.clone(), source_feed);

            Ok(())
        }
    }

    #[test]
    fn verify_ping_pong() {
        let storage = PodRocketStorageTest::new();

        let rocket = make_rocket(Arc::new(storage));
        let client = Client::new(rocket).expect("not a valid rocket instance");
        let mut res = client.get("/ping").dispatch();

        assert_eq!(res.status(), Status::Ok);

        let res_body = res.body_string().unwrap();
        assert_eq!("pong".to_string(), res_body);
    }

    #[test]
    fn verify_get_all_source_feeds() {
        let storage = PodRocketStorageTest::new();

        let source1 = SourceFeed::new(SOURCE1, "");
        let source2 = SourceFeed::new(SOURCE2, "");
        let source3 = SourceFeed::new(SOURCE3, "");

        let original_source_feeds = vec![source1.clone(), source2.clone(), source3.clone()];

        storage.add_source_feed(source1).unwrap();
        storage.add_source_feed(source2).unwrap();
        storage.add_source_feed(source3).unwrap();

        let rocket = make_rocket(Arc::new(storage));
        let client = Client::new(rocket).expect("not a valid rocket instance");
        let mut res = client.get(SOURCE_FEED_URL).dispatch();

        assert_eq!(res.status(), Status::Ok);

        let res_body = res.body_string().unwrap();
        let source_feeds: Vec<SourceFeed> = serde_json::from_str(&res_body).unwrap();

        let mut found_feeds = 0;
        for orig_feed in &original_source_feeds {
            for new_feed in &source_feeds {
                if orig_feed.url == new_feed.url {
                    found_feeds += 1;
                    break;
                }
            }
        }
        assert_eq!(original_source_feeds.len(), found_feeds);

        // TODO: better tests ?
        assert!(res_body.contains(SOURCE1));
        assert!(res_body.contains(SOURCE2));
        assert!(res_body.contains(SOURCE3));
    }

    #[test]
    fn verify_get_source_feed() {
        let storage = Arc::new(PodRocketStorageTest::new());

        let source1 = SourceFeed::new(SOURCE1, "");
        let source2 = SourceFeed::new(SOURCE2, "");
        let source3 = SourceFeed::new(SOURCE3, "");

        let rocket = make_rocket(storage.clone());
        let client = Client::new(rocket).expect("not a valid rocket instance");

        let res = client
            .get(format!("{}/{}", SOURCE_FEED_URL, SOURCE1))
            .dispatch();
        assert_eq!(res.status(), Status::NotFound);

        storage.add_source_feed(source1.clone()).unwrap();
        storage.add_source_feed(source2.clone()).unwrap();
        storage.add_source_feed(source3.clone()).unwrap();

        let mut res = client
            .get(format!("{}/{}", SOURCE_FEED_URL, SOURCE1))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let source_feed: SourceFeed = serde_json::from_str(&res_body).unwrap();
        assert!(res_body.contains(SOURCE1));
        assert_eq!(source1.url, source_feed.url);

        let mut res = client
            .get(format!("{}/{}", SOURCE_FEED_URL, SOURCE2))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let source_feed: SourceFeed = serde_json::from_str(&res_body).unwrap();
        assert!(res_body.contains(SOURCE2));
        assert_eq!(source2.url, source_feed.url);

        let mut res = client
            .get(format!("{}/{}", SOURCE_FEED_URL, SOURCE3))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let source_feed: SourceFeed = serde_json::from_str(&res_body).unwrap();
        assert!(res_body.contains(SOURCE3));
        assert_eq!(source3.url, source_feed.url);
    }

    #[test]
    fn verify_post_source_feed() {
        let storage = Arc::new(PodRocketStorageTest::new());

        let source1 = SourceFeed::new(SOURCE1, "");
        let source2 = SourceFeed::new(SOURCE2, "");
        let source3 = SourceFeed::new(SOURCE3, "");

        let rocket = make_rocket(storage.clone());
        let client = Client::new(rocket).expect("not a valid rocket instance");

        let res = client
            .post(SOURCE_FEED_URL)
            .header(ContentType::JSON)
            .body(serde_json::to_string(&source1).unwrap())
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(storage.get_source_feed_by_url(SOURCE1).unwrap().is_some());
        assert_eq!(
            storage
                .get_source_feed_by_url(SOURCE1)
                .unwrap()
                .unwrap()
                .url,
            source1.url
        );

        let res = client
            .post(SOURCE_FEED_URL)
            .header(ContentType::JSON)
            .body(serde_json::to_string(&source2).unwrap())
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(storage.get_source_feed_by_url(SOURCE2).unwrap().is_some());
        assert_eq!(
            storage
                .get_source_feed_by_url(SOURCE2)
                .unwrap()
                .unwrap()
                .url,
            source2.url
        );

        let res = client
            .post(SOURCE_FEED_URL)
            .header(ContentType::JSON)
            .body(serde_json::to_string(&source3).unwrap())
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        assert!(storage.get_source_feed_by_url(SOURCE3).unwrap().is_some());
        assert_eq!(
            storage
                .get_source_feed_by_url(SOURCE3)
                .unwrap()
                .unwrap()
                .url,
            source3.url
        );
    }

    #[test]
    fn verify_get_rss_feeds_by_id() {
        let storage = Arc::new(PodRocketStorageTest::new());

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash().to_string();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash().to_string();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash().to_string();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash().to_string();

        let rocket = make_rocket(storage.clone());
        let client = Client::new(rocket).expect("not a valid rocket instance");

        let res = client
            .get(format!("{}/{}", RSS_FEED_URL, SOURCE1))
            .dispatch();
        assert_eq!(res.status(), Status::NotFound);

        storage.add_new_rss_feed(feed1.clone()).unwrap();
        storage.add_new_rss_feed(feed2.clone()).unwrap();
        storage.add_new_rss_feed(feed3.clone()).unwrap();
        storage.add_new_rss_feed(feed4.clone()).unwrap();

        let mut res = client
            .get(format!("{}/{}", RSS_FEED_URL, feed1_hash))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feed: RssFeed = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feed.get_hash(), feed1_hash);
        assert_eq!(feed.get_items().len(), feed1.get_items().len());
        let mut found_items = 0;
        for orig_item in feed1.get_items() {
            for new_item in feed.get_items() {
                if new_item.get_guid() == orig_item.get_guid() {
                    found_items += 1;
                    break;
                }
            }
        }
        assert_eq!(found_items, feed1.get_items().len());

        let mut res = client
            .get(format!("{}/{}", RSS_FEED_URL, feed2_hash))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feed: RssFeed = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feed.get_hash(), feed2_hash);
        assert_eq!(feed.get_items().len(), feed2.get_items().len());
        let mut found_items = 0;
        for orig_item in feed2.get_items() {
            for new_item in feed.get_items() {
                if new_item.get_guid() == orig_item.get_guid() {
                    found_items += 1;
                    break;
                }
            }
        }
        assert_eq!(found_items, feed2.get_items().len());

        let mut res = client
            .get(format!("{}/{}", RSS_FEED_URL, feed3_hash))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feed: RssFeed = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feed.get_hash(), feed3_hash);
        assert_eq!(feed.get_items().len(), feed3.get_items().len());
        let mut found_items = 0;
        for orig_item in feed3.get_items() {
            for new_item in feed.get_items() {
                if new_item.get_guid() == orig_item.get_guid() {
                    found_items += 1;
                    break;
                }
            }
        }
        assert_eq!(found_items, feed3.get_items().len());

        let mut res = client
            .get(format!("{}/{}", RSS_FEED_URL, feed4_hash))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feed: RssFeed = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feed.get_hash(), feed4_hash);
        assert_eq!(feed.get_items().len(), feed4.get_items().len());
        let mut found_items = 0;
        for orig_item in feed4.get_items() {
            for new_item in feed.get_items() {
                if new_item.get_guid() == orig_item.get_guid() {
                    found_items += 1;
                    break;
                }
            }
        }
        assert_eq!(found_items, feed4.get_items().len());
    }

    #[test]
    fn verify_get_rss_feeds_by_url() {
        let storage = Arc::new(PodRocketStorageTest::new());

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash().to_string();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash().to_string();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash().to_string();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash().to_string();

        let rocket = make_rocket(storage.clone());
        let client = Client::new(rocket).expect("not a valid rocket instance");

        let mut res = client
            .get(format!("{}/url/{}", RSS_FEED_URL, SOURCE1))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feeds: Vec<RssFeed> = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feeds.len(), 0);

        storage.add_new_rss_feed(feed1.clone()).unwrap();

        let mut res = client
            .get(format!("{}/url/{}", RSS_FEED_URL, SOURCE1))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feeds: Vec<RssFeed> = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feeds.len(), 1);
        let feed = feeds[0].clone();
        assert_eq!(feed.get_hash(), feed1_hash);
        assert_eq!(feed.get_items().len(), feed1.get_items().len());
        let mut found_items = 0;
        for orig_item in feed1.get_items() {
            for new_item in feed.get_items() {
                if new_item.get_guid() == orig_item.get_guid() {
                    found_items += 1;
                    break;
                }
            }
        }
        assert_eq!(found_items, feed1.get_items().len());

        storage.add_new_rss_feed(feed2.clone()).unwrap();

        let mut res = client
            .get(format!("{}/url/{}", RSS_FEED_URL, SOURCE2))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feeds: Vec<RssFeed> = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feeds.len(), 1);
        let feed = feeds[0].clone();
        assert_eq!(feed.get_hash(), feed2_hash);
        assert_eq!(feed.get_items().len(), feed2.get_items().len());
        let mut found_items = 0;
        for orig_item in feed2.get_items() {
            for new_item in feed.get_items() {
                if new_item.get_guid() == orig_item.get_guid() {
                    found_items += 1;
                    break;
                }
            }
        }
        assert_eq!(found_items, feed2.get_items().len());

        storage.add_new_rss_feed(feed3.clone()).unwrap();

        let mut res = client
            .get(format!("{}/url/{}", RSS_FEED_URL, SOURCE1))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feeds: Vec<RssFeed> = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feeds.len(), 2);
        let feed = feeds[1].clone();
        assert_eq!(feed.get_hash(), feed3_hash);
        assert_eq!(feed.get_items().len(), feed3.get_items().len());
        let mut found_items = 0;
        for orig_item in feed3.get_items() {
            for new_item in feed.get_items() {
                if new_item.get_guid() == orig_item.get_guid() {
                    found_items += 1;
                    break;
                }
            }
        }
        assert_eq!(found_items, feed3.get_items().len());

        storage.add_new_rss_feed(feed4.clone()).unwrap();

        let mut res = client
            .get(format!("{}/url/{}", RSS_FEED_URL, SOURCE2))
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feeds: Vec<RssFeed> = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feeds.len(), 2);
        let feed = feeds[1].clone();
        assert_eq!(feed.get_hash(), feed4_hash);
        assert_eq!(feed.get_items().len(), feed4.get_items().len());
        let mut found_items = 0;
        for orig_item in feed4.get_items() {
            for new_item in feed.get_items() {
                if new_item.get_guid() == orig_item.get_guid() {
                    found_items += 1;
                    break;
                }
            }
        }
        assert_eq!(found_items, feed4.get_items().len());
    }

    fn contains_hash(vector_to_check: &Vec<RssFeed>, hash: String) -> bool {
        for feed in vector_to_check {
            if feed.get_hash() == hash {
                return true;
            }
        }
        false
    }

    #[test]
    fn verify_get_all_rss() {
        let storage = Arc::new(PodRocketStorageTest::new());

        let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        let feed1_hash = feed1.get_hash().to_string();
        let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        let feed2_hash = feed2.get_hash().to_string();
        let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        let feed3_hash = feed3.get_hash().to_string();
        let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        let feed4_hash = feed4.get_hash().to_string();

        let rocket = make_rocket(storage.clone());
        let client = Client::new(rocket).expect("not a valid rocket instance");

        let mut res = client.get(format!("{}", RSS_FEED_URL)).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feeds: HashMap<String, Vec<RssFeed>> = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feeds.len(), 0);

        storage.add_new_rss_feed(feed1.clone()).unwrap();

        let mut res = client.get(format!("{}", RSS_FEED_URL)).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feeds: HashMap<String, Vec<RssFeed>> = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feeds.get(SOURCE1).unwrap().len(), 1);
        assert!(feeds.get(SOURCE2).is_none());
        let mut found_feeds = vec![];
        for values in feeds.values() {
            for a_found_feed in values {
                found_feeds.push(a_found_feed.clone());
            }
        }
        assert_eq!(found_feeds.len(), 1);
        assert!(contains_hash(&found_feeds, feed1_hash));

        storage.add_new_rss_feed(feed2.clone()).unwrap();

        let mut res = client.get(format!("{}", RSS_FEED_URL)).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feeds: HashMap<String, Vec<RssFeed>> = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feeds.get(SOURCE1).unwrap().len(), 1);
        assert_eq!(feeds.get(SOURCE2).unwrap().len(), 1);
        let mut found_feeds = vec![];
        for values in feeds.values() {
            for a_found_feed in values {
                found_feeds.push(a_found_feed.clone());
            }
        }
        assert_eq!(found_feeds.len(), 2);
        assert!(contains_hash(&found_feeds, feed2_hash));

        storage.add_new_rss_feed(feed3.clone()).unwrap();

        let mut res = client.get(format!("{}", RSS_FEED_URL)).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feeds: HashMap<String, Vec<RssFeed>> = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feeds.get(SOURCE1).unwrap().len(), 2);
        assert_eq!(feeds.get(SOURCE2).unwrap().len(), 1);
        let mut found_feeds = vec![];
        for values in feeds.values() {
            for a_found_feed in values {
                found_feeds.push(a_found_feed.clone());
            }
        }
        assert_eq!(found_feeds.len(), 3);
        assert!(contains_hash(&found_feeds, feed3_hash));

        storage.add_new_rss_feed(feed4.clone()).unwrap();

        let mut res = client.get(format!("{}", RSS_FEED_URL)).dispatch();
        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        let feeds: HashMap<String, Vec<RssFeed>> = serde_json::from_str(&res_body).unwrap();
        assert_eq!(feeds.get(SOURCE1).unwrap().len(), 2);
        assert_eq!(feeds.get(SOURCE2).unwrap().len(), 2);
        let mut found_feeds = vec![];
        for values in feeds.values() {
            for a_found_feed in values {
                found_feeds.push(a_found_feed.clone());
            }
        }
        assert_eq!(found_feeds.len(), 4);
        assert!(contains_hash(&found_feeds, feed4_hash));
    }
}

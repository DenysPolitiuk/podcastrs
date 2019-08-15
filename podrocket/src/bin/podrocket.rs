#![feature(proc_macro_hygiene, decl_macro)]
use rocket::Data;
use rocket::Rocket;
use rocket::State;
use rocket::{get, post, routes};
use rocket_contrib::json::Json;

use std::sync::Arc;

use common::SourceFeed;
use podrocket_trait::PodRocketStorage;
use storage::RssStorage;
use storage::RssStorageConfig;

// TODO:
//  Source Feed:
//      * GET -> get all source feeds
//      * GET by id -> get one source feed
//      * POST -> add new source feed
//  Rss Feed:
//      * GET -> get all rss feeds (limit the amount ?)
//      * GET by id -> get one rss feed
//      * GET by source feed -> get all rss feeds by a given source feed (limit the amount ?)

static DEFAULT_HOST: &str = "localhost";
static DEFAULT_PORT: u16 = 27017;

type Storage = dyn PodRocketStorage + Send + Sync;

#[get("/source")]
fn get_all_source_feeds(storage: State<Arc<Storage>>) -> Option<Json<Vec<SourceFeed>>> {
    let source_feeds = storage
        .get_source_feeds()
        .expect("unable to get source feeds");

    let source_feeds_vec = source_feeds
        .values()
        .into_iter()
        .map(|sf| sf.clone())
        .collect();

    Some(Json(source_feeds_vec))
}

#[get("/source/<id>")]
fn get_source_feed(id: String) -> String {
    format!("Hello from get {} source feed", id)
}

#[post("/source", data = "<data>")]
fn post_source_feed(data: Data) -> String {
    "Hello from post source feed".into()
}

#[get("/rss")]
fn get_all_rss_feeds() -> String {
    "Hello from get all rss feeds".into()
}

#[get("/rss/<id>", rank = 1)]
fn get_rss_feeds_by_id(id: usize) -> String {
    format!("Hello from get id {:?} rss feed", id)
}

#[get("/rss/<url>", rank = 2)]
fn get_rss_feeds_by_url(url: String) -> String {
    format!("Hello from get url {:?} rss feed", url)
}

fn make_rocket(database_client: Arc<Storage>) -> Rocket {
    rocket::ignite().manage(database_client).mount(
        "/",
        routes![
            get_all_source_feeds,
            get_source_feed,
            post_source_feed,
            get_all_rss_feeds,
            get_rss_feeds_by_id,
            get_rss_feeds_by_url,
        ],
    )
}

fn main() {
    let storage = RssStorage::new(DEFAULT_HOST, DEFAULT_PORT).expect("unable to create storate");
    make_rocket(Arc::new(storage)).launch();
}

#[cfg(test)]
mod tests {
    use super::*;

    use rocket::http::Status;
    use rocket::local::Client;

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
    fn verify_get_all_source_feeds() {
        let storage = PodRocketStorageTest::new();

        let source1 = SourceFeed::new(SOURCE1, "");
        let source2 = SourceFeed::new(SOURCE2, "");
        let source3 = SourceFeed::new(SOURCE3, "");

        storage.add_source_feed(source1).unwrap();
        storage.add_source_feed(source2).unwrap();
        storage.add_source_feed(source3).unwrap();

        let rocket = make_rocket(Arc::new(storage));
        let client = Client::new(rocket).expect("not a valid rocket instance");
        let mut res = client.get(SOURCE_FEED_URL).dispatch();

        assert_eq!(res.status(), Status::Ok);
        let res_body = res.body_string().unwrap();
        // TODO: better tests ?
        assert!(res_body.contains(SOURCE1));
        assert!(res_body.contains(SOURCE2));
        assert!(res_body.contains(SOURCE3));
    }

    #[test]
    fn verify_get_source_feed() {
        panic!("unimplemented");
    }

    #[test]
    fn verify_post_source_feed() {
        panic!("unimplemented");
    }

    #[test]
    fn verify_get_all_rss_feeds() {
        // let storage =
        // RssStorage::new(DEFAULT_HOST, DEFAULT_PORT).expect("unable to create storage");

        // let feed1 = RssFeed::new_from_file(SOURCE1, FEED1_FILE).unwrap();
        // let feed1_hash = feed1.get_hash().to_string();
        // let feed2 = RssFeed::new_from_file(SOURCE2, FEED2_FILE).unwrap();
        // let feed2_hash = feed2.get_hash().to_string();
        // let feed3 = RssFeed::new_from_file(SOURCE1, FEED2_FILE).unwrap();
        // let feed3_hash = feed3.get_hash().to_string();
        // let feed4 = RssFeed::new_from_file(SOURCE2, FEED1_FILE).unwrap();
        // let feed4_hash = feed4.get_hash().to_string();

        // let rocket = make_rocket();
        // let client = Client::new(rocket).expect("not a valid rocket instance");
        // let mut res = client.get(RSS_FEED_URL).dispatch();

        // assert_eq!(res.status(), Status::Ok);
        // let res_body = res.body_string().unwrap();

        panic!("not finished");
    }

    #[test]
    fn verify_get_rss_feeds_by_id() {
        panic!("unimplemented");
    }

    #[test]
    fn verify_get_rss_feeds_by_url() {
        panic!("unimplemented");
    }
}

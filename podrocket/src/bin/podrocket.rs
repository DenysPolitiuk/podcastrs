#![feature(proc_macro_hygiene, decl_macro)]
use rocket::Data;
use rocket::Rocket;
use rocket::{get, post, routes};

// TODO:
//  Source Feed:
//      * GET -> get all source feeds
//      * GET by id -> get one source feed
//      * POST -> add new source feed
//  Rss Feed:
//      * GET -> get all rss feeds (limit the amount ?)
//      * GET by id -> get one rss feed
//      * GET by source feed -> get all rss feeds by a given source feed (limit the amount ?)

#[get("/source")]
fn get_all_source_feeds() -> String {
    "Hello from get all source feeds".into()
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

fn make_rocket() -> Rocket {
    rocket::ignite().mount(
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
    make_rocket().launch();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_get_all_source_feeds() {
        panic!("unimplemented");
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
        panic!("unimplemented");
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

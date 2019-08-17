use std::env;

use scheduler::RssScheduler;
use storage::RssStorage;

static DEFAULT_HOST: &str = "localhost";
static DEFAULT_PORT: u16 = 27017;

static HOST_ENV_VAR: &str = "RSS_DATABASE_HOST";
static PORT_ENV_VAR: &str = "RSS_DATABASE_PORT";

fn main() {
    let storage = match RssStorage::new(
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
    ) {
        Ok(v) => v,
        Err(e) => {
            println!("Error during storage creation : {}", e);
            return;
        }
    };

    let mut scheduler = RssScheduler::new();
    scheduler
        .do_work(None, &storage)
        .iter()
        .for_each(|e| println!("Error from do work : {}", e));
}

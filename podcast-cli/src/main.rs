use common::SourceFeed;
use storage::RssStorage;

use clap::{App, Arg};
use serde_json;

use std::env;
use std::fs::File;
use std::io::prelude::*;

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

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::with_name("add-source")
                .short("S")
                .long("add-source")
                .takes_value(true)
                .help("add source feed"),
        )
        .get_matches();

    if let Some(file_name) = matches.value_of("add-source") {
        import_source_feeds(file_name, &storage);
    }
}

fn import_source_feeds(input_file_name: &str, storage: &RssStorage) {
    let mut json_content = String::new();
    let mut f = File::open(input_file_name).expect("unable to open provided file");

    f.read_to_string(&mut json_content)
        .expect("unable to read provided file");
    let source_feeds: Vec<SourceFeed> =
        serde_json::from_str(&json_content).expect("unable to parse json");

    for sfeed in source_feeds {
        if let Err(e) = storage.add_source_feed(sfeed) {
            println!("Error storing source feed to storage : {}", e);
        }
    }
}

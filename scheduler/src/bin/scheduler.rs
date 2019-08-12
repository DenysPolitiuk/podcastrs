use common::SourceFeed;
use scheduler::RssScheduler;
use scheduler_trait::RssSchedulerStorage;
use storage::RssStorage;

static DEFAULT_HOST: &str = "localhost";
static DEFAULT_PORT: u16 = 27017;

fn main() {
    let storage = match RssStorage::new(DEFAULT_HOST, DEFAULT_PORT) {
        Ok(v) => v,
        Err(e) => {
            println!("Error during storage creation : {}", e);
            return;
        }
    };
    let default_feed_sources = vec!["https://softwareengineeringdaily.com/category/podcast/feed"];
    for feed_source in default_feed_sources {
        if let Err(e) = storage.add_source_feed(SourceFeed::new(feed_source)) {
            println!(
                "Error adding source feed {} to storage : {}",
                feed_source, e
            );
        }
    }

    let mut scheduler = RssScheduler::new();
    scheduler
        .do_work("download", &storage)
        .iter()
        .for_each(|e| println!("Error from do work : {}", e));
}

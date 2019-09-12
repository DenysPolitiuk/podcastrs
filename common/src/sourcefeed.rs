use serde::{Deserialize, Serialize};
use serde_json;

use util;

use std::error::Error;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SourceFeed {
    pub url: String,
    title: String,
    // TODO: change back to u32 after using new datastore
    hash: Option<i64>,
}

impl SourceFeed {
    fn new_no_hash(url: &str, title: &str) -> SourceFeed {
        SourceFeed {
            url: url.to_string(),
            title: title.to_string(),
            hash: None,
        }
    }

    pub fn new(url: &str, title: &str) -> Result<SourceFeed, Box<dyn Error>> {
        SourceFeed::new_no_hash(url, title).with_compute_hash()
    }

    pub fn with_compute_hash(self) -> Result<Self, Box<dyn Error>> {
        let json = serde_json::to_string(&self)?;

        Ok(SourceFeed {
            url: self.url.clone(),
            title: self.title.clone(),
            hash: Some(i64::from(util::compute_hash(&json))),
        })
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn get_hash(&self) -> i64 {
        match self.hash {
            Some(v) => v,
            None => self
                .clone()
                .with_compute_hash()
                .expect("internal hashing error")
                .hash
                .unwrap(),
        }
    }
}

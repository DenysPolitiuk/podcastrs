use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SourceFeed {
    pub url: String,
    title: String,
}

impl SourceFeed {
    pub fn new(url: &str, title: &str) -> SourceFeed {
        SourceFeed {
            url: url.to_string(),
            title: title.to_string(),
        }
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SourceFeed {
    pub url: String,
}

impl SourceFeed {
    #[allow(dead_code)]
    pub fn new(url: &str) -> SourceFeed {
        SourceFeed {
            url: url.to_string(),
        }
    }
}

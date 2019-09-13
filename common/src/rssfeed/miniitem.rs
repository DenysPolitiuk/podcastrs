use rss::{Enclosure, Guid, Item};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct MiniItem {
    title: Option<String>,
    link: Option<String>,
    pub_date: Option<String>,
    guid: Option<Guid>,
    enclosure: Option<Enclosure>,
    description: Option<String>,
}

impl MiniItem {
    pub fn from_item(item: &Item) -> MiniItem {
        MiniItem {
            title: item.title().map(|s| s.to_string()),
            link: item.link().map(|s| s.to_string()),
            pub_date: item.pub_date().map(|s| s.to_string()),
            guid: item.guid().cloned(),
            enclosure: item.enclosure().cloned(),
            description: item.description().map(|s| s.to_string()),
        }
    }

    pub fn get_title(&self) -> Option<&str> {
        self.title.as_ref().map(String::as_ref)
    }

    pub fn get_guid(&self) -> Option<&str> {
        self.guid.as_ref().map(|g| &*g.value())
    }

    pub fn get_enclosure_url(&self) -> Option<&str> {
        self.enclosure.as_ref().map(|e| &*e.url())
    }
}

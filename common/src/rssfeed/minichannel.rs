use rss::Channel;
use serde::{Deserialize, Serialize};

use super::MiniItem;

#[derive(Clone, Deserialize, Serialize)]
pub struct MiniChannel {
    title: String,
    description: String,
    link: String,
    last_build_date: Option<String>,
    items: Vec<MiniItem>,
}

impl MiniChannel {
    pub fn from_channel(channel: &Channel) -> MiniChannel {
        MiniChannel {
            title: channel.title().to_string(),
            description: channel.description().to_string(),
            link: channel.link().to_string(),
            last_build_date: channel.last_build_date().map(|s| s.to_string()),
            items: channel
                .items()
                .iter()
                .map(|i| MiniItem::from_item(i))
                .collect(),
        }
    }

    pub fn get_items(&self) -> &Vec<MiniItem> {
        &self.items
    }
}

use rss::Enclosure;
use rss::Guid;
use rss::Item;
use serde::{Deserialize, Serialize};

use super::super::BasicMeta;
use super::RssCategory;

use std::error::Error;

type ErrorType = dyn Error + Send + Sync;

#[derive(Clone, Deserialize, Serialize)]
pub struct RssItem {
    title: Option<String>,
    link: Option<String>,
    description: Option<String>,
    categories: Vec<RssCategory>,
    comments: Option<String>,
    enclosure: Option<Enclosure>,
    guid: Option<Guid>,
    pub_date: Option<String>,
    metadata: BasicMeta,
}

impl RssItem {
    fn new_no_hash(item: &Item) -> RssItem {
        RssItem {
            title: item.title().map(|s| s.to_string()),
            link: item.link().map(|s| s.to_string()),
            description: item.description().map(|s| s.to_string()),
            // TODO: add categories
            categories: item
                .categories()
                .iter()
                .map(|c| RssCategory::new_from_category(c))
                .filter_map(|c| c.ok())
                .collect(),
            comments: item.comments().map(|s| s.to_string()),
            enclosure: item.enclosure().cloned(),
            guid: item.guid().cloned(),
            pub_date: item.pub_date().map(|s| s.to_string()),
            metadata: BasicMeta::new(),
        }
    }

    fn with_compute_hash(self) -> Result<Self, Box<ErrorType>> {
        let meta = self.metadata.clone();

        Ok(RssItem {
            title: self.title.clone(),
            link: self.link.clone(),
            description: self.description.clone(),
            categories: self.categories.clone(),
            comments: self.comments.clone(),
            enclosure: self.enclosure.clone(),
            guid: self.guid.clone(),
            pub_date: self.pub_date.clone(),
            metadata: meta.with_compute_hash(&self)?,
        })
    }

    pub fn new_from_item(item: &Item) -> Result<RssItem, Box<ErrorType>> {
        RssItem::new_no_hash(item).with_compute_hash()
    }

    pub fn get_title(&self) -> Option<String> {
        self.title.clone()
    }

    pub fn get_link(&self) -> Option<String> {
        self.link.clone()
    }

    pub fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn get_categories(&self) -> &[RssCategory] {
        &self.categories
    }

    pub fn get_comments(&self) -> Option<String> {
        self.comments.clone()
    }

    pub fn get_enclosure(&self) -> &Option<Enclosure> {
        &self.enclosure
    }

    pub fn get_enclosure_url(&self) -> Option<&str> {
        self.get_enclosure().as_ref().map(|e| &*e.url())
    }

    pub fn get_guid(&self) -> &Option<Guid> {
        &self.guid
    }

    pub fn get_guid_value(&self) -> Option<&str> {
        self.get_guid().as_ref().map(|g| &*g.value())
    }

    pub fn get_pub_date(&self) -> Option<String> {
        self.pub_date.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use rss::Channel;

    use std::fs::File;
    use std::io::BufReader;

    static TEST_FEED: &str = "../tests/sedaily.rss";

    impl RssItem {
        fn change_enclosure_url(&mut self, url: &str) {
            if let Some(enclosure) = &mut self.enclosure {
                enclosure.set_url(url);
            }
        }

        fn change_guid_value(&mut self, guid_value: &str) {
            if let Some(guid) = &mut self.guid {
                guid.set_value(guid_value);
            }
        }
    }

    fn create_channel_from_file(name: &str) -> Channel {
        let file = File::open(name).unwrap();
        let buf_reader = BufReader::new(&file);
        Channel::read_from(buf_reader).unwrap()
    }

    fn create_item(feed_name: &str) -> RssItem {
        let channel = create_channel_from_file(feed_name);
        RssItem::new_from_item(&channel.items()[0]).unwrap()
    }

    #[test]
    fn create() {
        let _ = create_item(TEST_FEED);
    }

    #[test]
    fn get_title_none() {
        let mut item = create_item(TEST_FEED);
        item.title = None;
        assert!(item.get_title().is_none());
    }

    #[test]
    fn get_title_some() {
        let mut item = create_item(TEST_FEED);
        let text = Some("this is title".to_string());
        item.title = text.clone();
        assert!(item.get_title().is_some());
        assert_eq!(text, item.get_title());
    }

    #[test]
    fn get_link_none() {
        let mut item = create_item(TEST_FEED);
        item.link = None;
        assert!(item.get_link().is_none());
    }

    #[test]
    fn get_link_some() {
        let mut item = create_item(TEST_FEED);
        let text = Some("this is link".to_string());
        item.link = text.clone();
        assert!(item.get_link().is_some());
        assert_eq!(text, item.get_link());
    }

    #[test]
    fn get_description_none() {
        let mut item = create_item(TEST_FEED);
        item.description = None;
        assert!(item.get_description().is_none());
    }

    #[test]
    fn get_description_some() {
        let mut item = create_item(TEST_FEED);
        let text = Some("this is description".to_string());
        item.description = text.clone();
        assert!(item.get_description().is_some());
        assert_eq!(text, item.get_description());
    }

    #[test]
    fn get_categories_empty() {
        let mut item = create_item(TEST_FEED);
        item.categories = vec![];
        assert!(item.get_categories().is_empty());
    }

    #[test]
    fn get_categories_with_data() {
        let item = create_item(TEST_FEED);
        assert!(!item.get_categories().is_empty());
    }

    #[test]
    fn get_comments_none() {
        let mut item = create_item(TEST_FEED);
        item.comments = None;
        assert!(item.get_comments().is_none());
    }

    #[test]
    fn get_comments_some() {
        let mut item = create_item(TEST_FEED);
        let text = Some("this is comments".to_string());
        item.comments = text.clone();
        assert!(item.get_comments().is_some());
        assert_eq!(text, item.get_comments());
    }

    #[test]
    fn get_enclosure_none() {
        let mut item = create_item(TEST_FEED);
        item.enclosure = None;
        assert!(item.get_enclosure().is_none());
    }

    #[test]
    fn get_enclosure_some() {
        let item = create_item(TEST_FEED);
        assert!(item.get_enclosure().is_some());
    }

    #[test]
    fn get_enclosure_url_some() {
        let mut item = create_item(TEST_FEED);
        let text = "this is url";
        item.change_enclosure_url(text);
        assert!(item.get_enclosure_url().is_some());
        assert_eq!(Some(text), item.get_enclosure_url());
    }

    #[test]
    fn get_guid_none() {
        let mut item = create_item(TEST_FEED);
        item.guid = None;
        assert!(item.get_guid().is_none());
    }

    #[test]
    fn get_guid_some() {
        let item = create_item(TEST_FEED);
        assert!(item.get_guid().is_some());
    }

    #[test]
    fn get_guid_value_some() {
        let mut item = create_item(TEST_FEED);
        let text = "this is guid";
        item.change_guid_value(text);
        assert!(item.get_guid_value().is_some());
        assert_eq!(item.get_guid_value(), Some(text));
    }

    #[test]
    fn get_pub_date_none() {
        let mut item = create_item(TEST_FEED);
        item.pub_date = None;
        assert!(item.get_pub_date().is_none());
    }

    #[test]
    fn get_pub_date_some() {
        let mut item = create_item(TEST_FEED);
        let text = Some("this is pub date".to_string());
        item.pub_date = text.clone();
        assert!(item.get_pub_date().is_some());
        assert_eq!(text, item.get_pub_date());
    }
}

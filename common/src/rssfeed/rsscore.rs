use rss::Channel;
use rss::Image;
use serde::{Deserialize, Serialize};

use super::super::BasicMeta;

use std::error::Error;

#[derive(Clone, Deserialize, Serialize)]
pub struct RssFeedCore {
    title: String,
    link: String,
    description: String,
    pub_date: Option<String>,
    last_build_date: Option<String>,
    image: Option<Image>,
    // TODO: replace with u32
    metadata: BasicMeta,
}

impl RssFeedCore {
    fn new_no_hash(channel: &Channel) -> RssFeedCore {
        RssFeedCore {
            title: channel.title().to_string(),
            link: channel.link().to_string(),
            description: channel.description().to_string(),
            pub_date: channel.pub_date().map(|s| s.to_string()),
            last_build_date: channel.last_build_date().map(|s| s.to_string()),
            image: channel.image().cloned(),
            metadata: BasicMeta::new(),
        }
    }

    pub fn new_from_channel(channel: &Channel) -> Result<RssFeedCore, Box<dyn Error>> {
        RssFeedCore::new_no_hash(channel).with_compute_hash()
    }

    fn with_compute_hash(self) -> Result<Self, Box<dyn Error>> {
        let meta = self.metadata.clone();

        Ok(RssFeedCore {
            title: self.title.clone(),
            link: self.link.clone(),
            description: self.description.clone(),
            pub_date: self.pub_date.clone(),
            last_build_date: self.last_build_date.clone(),
            image: self.image.clone(),
            metadata: meta.with_compute_hash(&self)?,
        })
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }

    pub fn get_link(&self) -> String {
        self.link.clone()
    }

    pub fn get_description(&self) -> String {
        self.description.clone()
    }

    pub fn get_pub_date(&self) -> Option<String> {
        self.pub_date.clone()
    }

    pub fn get_last_build_date(&self) -> Option<String> {
        self.last_build_date.clone()
    }

    pub fn get_image(&self) -> &Option<Image> {
        &self.image
    }

    pub fn get_timestamp(&self) -> i64 {
        self.metadata.get_timestamp()
    }

    pub fn get_hash(&self) -> i64 {
        match self.metadata.get_hash() {
            Some(v) => v,
            None => self
                .clone()
                .with_compute_hash()
                .expect("internal hashing error")
                .get_hash(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rss::Channel;

    use std::fs::File;
    use std::io::BufReader;

    static TEST_FEED: &str = "../tests/sedaily.rss";
    static TEST_FEED_2: &str = "../tests/sedaily2.rss";

    fn create_channel_from_file(name: &str) -> Channel {
        let file = File::open(name).unwrap();
        let buf_reader = BufReader::new(&file);
        Channel::read_from(buf_reader).unwrap()
    }

    fn create_core(feed_name: &str) -> RssFeedCore {
        let channel = create_channel_from_file(feed_name);
        RssFeedCore::new_from_channel(&channel).unwrap()
    }

    #[test]
    fn create() {
        let _ = create_core(TEST_FEED);
    }

    #[test]
    fn get_title() {
        let mut core = create_core(TEST_FEED);
        let title = "this is title";
        core.title = title.to_string();
        assert_eq!(title, core.get_title());
    }

    #[test]
    fn get_link() {
        let mut core = create_core(TEST_FEED);
        let link = "this is link";
        core.link = link.to_string();
        assert_eq!(link, core.get_link());
    }

    #[test]
    fn get_description() {
        let mut core = create_core(TEST_FEED);
        let text = "this is description";
        core.description = text.to_string();
        assert_eq!(text, core.get_description());
    }

    #[test]
    fn get_pub_date_none() {
        let mut core = create_core(TEST_FEED);
        core.pub_date = None;
        assert!(core.get_pub_date().is_none());
    }

    #[test]
    fn get_pub_date_some() {
        let mut core = create_core(TEST_FEED);
        let text = "this is pub date";
        core.pub_date = Some(text.to_string());
        assert!(core.get_pub_date().is_some());
        assert_eq!(Some(text.to_string()), core.get_pub_date());
    }

    #[test]
    fn get_last_build_date_none() {
        let mut core = create_core(TEST_FEED);
        core.last_build_date = None;
        assert!(core.get_last_build_date().is_none());
    }

    #[test]
    fn get_last_build_date_some() {
        let mut core = create_core(TEST_FEED);
        let text = "this is build date";
        core.last_build_date = Some(text.to_string());
        assert!(core.get_last_build_date().is_some());
        assert_eq!(Some(text.to_string()), core.get_last_build_date());
    }

    #[test]
    fn get_image_none() {
        let mut core = create_core(TEST_FEED);
        core.image = None;
        assert!(core.get_image().is_none());
    }

    #[test]
    fn get_image_some() {
        let core = create_core(TEST_FEED);
        assert!(core.get_image().is_some());
    }

    #[test]
    fn get_timestamp() {
        let mut core = create_core(TEST_FEED);
        core.metadata = core.metadata.with_timestamp(123);
        assert_eq!(core.get_timestamp(), 123);
    }

    #[test]
    fn get_hash() {
        let mut core = create_core(TEST_FEED);
        core.metadata = core.metadata.with_hash(123);
        assert_eq!(core.get_hash(), 123);
    }
}

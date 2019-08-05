use rss::Channel;
use std::fs::File;
use std::io::BufReader;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run() {
        let file = File::open("sedaily.rss").unwrap();
        let channel = Channel::read_from(BufReader::new(file)).unwrap();

        let items = channel.items();
        for item in items {
            let title = match item.title() {
                None => continue,
                Some(v) => v,
            };
            let enclosure = match item.enclosure() {
                None => continue,
                Some(v) => v,
            };
            println!("{} @ {}", title, enclosure.url());
        }

        println!("total number of items is {}", channel.items().len());
    }
}

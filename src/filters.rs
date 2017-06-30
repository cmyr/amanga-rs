use super::Tweet;

type Filter = fn(&Tweet) -> bool;

pub fn url_filter(tweet: &Tweet) -> bool {
    tweet.entities.urls.is_empty()
}

pub fn manual_url_filter(tweet: &Tweet) -> bool {
    tweet.text.find("https://t.co").is_none()
}

pub fn mention_filter(tweet: &Tweet) -> bool {
    tweet.entities.user_mentions.is_empty()
}

pub fn en_filter(tweet: &Tweet) -> bool {
    tweet.lang == "en"
}

/// Whether or not some percentage of characters are letters.
pub fn letterish(tweet: &Tweet) -> bool {
    let mut total_chars = 0;
    let mut letter_chars = 0;
    for chr in tweet.text.chars() {
        total_chars += 1;
        // ascii letters + space
        match u32::from(chr) {
            32 | 65 ... 91 | 97 ... 123 => letter_chars += 1,
            _ => (),
        }
    }
    letter_chars as f64 / total_chars as f64 >= 0.7
}

pub fn filter_all(tweet: &Tweet) -> bool {
    mention_filter(tweet) &&
    url_filter(tweet) &&
    en_filter(tweet) &&
    manual_url_filter(tweet) &&
    letterish(tweet)
}

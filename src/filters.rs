use gnip_twitter_stream::Tweet;

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
        if is_ascii_letter(&chr) { letter_chars += 1 }
        // ascii letters + space
    }
    letter_chars as f64 / total_chars as f64 >= 0.65
}

pub fn is_ascii_letter(chr: &char) -> bool {
    match *chr {
        'a' ... 'z' | 'A' ... 'Z' => true,
        _ => false,
    }
}

pub fn filter_all(tweet: &Tweet) -> bool {
    mention_filter(tweet) &&
        url_filter(tweet) &&
        en_filter(tweet) &&
    manual_url_filter(tweet) &&
    letterish(tweet)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_letter() {
        assert!(!is_ascii_letter('@'));
        assert!(!is_ascii_letter('['));
        assert!(!is_ascii_letter('`'));
        assert!(!is_ascii_letter('{'));
        assert!(is_ascii_letter('A'));
        assert!(is_ascii_letter('Z'));
        assert!(is_ascii_letter('a'));
        assert!(is_ascii_letter('z'));
    }
}

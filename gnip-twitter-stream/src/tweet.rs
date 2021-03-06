use chrono::{DateTime as ChronoDateTime, Utc};
pub type DateTime = ChronoDateTime<Utc>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tweet {
    #[serde(rename = "body")]
    pub text: String,
    #[serde(rename = "twitter_lang")]
    pub lang: String,
    pub link: String,
    #[serde(rename = "postedTime")]
    pub posted_time: DateTime,
    #[serde(rename = "actor")]
    pub user: User,
    #[serde(rename = "twitter_entities")]
    pub entities: Entities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entities {
    pub hashtags: Vec<Hashtag>,
    pub urls: Vec<Url>,
    pub user_mentions: Vec<UserMention>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub link: String,
    pub display_name: String,
    pub image: String,
    pub preferred_username: String,
    pub verified: bool,
    pub followers_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hashtag {
    pub text: String,
    pub indices: (u64, u64)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Url {
    pub url: String,
    pub expanded_url: String,
    pub indices: (u64, u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMention {
    pub screen_name: String,
    pub name: Option<String>,
    pub id: Option<u64>,
    pub id_str: Option<String>,
    pub indices: (u64, u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalTweet {
    #[serde(rename = "body")]
    pub text: String,
    pub link: String,
}

impl MinimalTweet {
    pub fn id(&self) -> u64 {
        self.link.split('/').last().and_then(|s| s.parse::<u64>().ok())
            .expect(&format!("failed to parse status {}", self.link))
    }
}

impl From<Tweet> for MinimalTweet {
    fn from(src: Tweet) -> MinimalTweet {
        MinimalTweet {
            text: src.text,
            link: src.link,
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tweet_id() {
        let t = MinimalTweet {
            text: "hello".into(),
            link: "http://twitter.com/jas_leigh/statuses/899903730065264640".into(),
        };
        assert_eq!(t.id(), 899903730065264640);
    }
}

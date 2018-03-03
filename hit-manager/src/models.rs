use std::time::SystemTime;
use std::io::Write;

use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Int4;

use super::schema::hits;
use super::schema::tweets;

#[derive(Debug, Copy, Clone, PartialEq, FromSqlRow, AsExpression, Serialize, Deserialize)]
#[sql_type = "Int4"]
#[serde(rename_all="snake_case")]
pub enum HitStatus {
    New = 0,
    FetchFailed = 1,
    Approved = 2,
    Rejected = 3,
    Posted = 4,
    PostFailed = 5,
}

impl ToSql<Int4, Pg> for HitStatus {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let val = *self as i32;
        <i32 as ToSql<Int4, Pg>>::to_sql(&val, out)
    }
}

impl FromSql<Int4, Pg> for HitStatus {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let as_int = <i32 as FromSql<Int4, Pg>>::from_sql(bytes)?;
        match as_int {
            0 => Ok(HitStatus::New),
            1 => Ok(HitStatus::FetchFailed),
            2 => Ok(HitStatus::Approved),
            3 => Ok(HitStatus::Rejected),
            4 => Ok(HitStatus::Posted),
            5 => Ok(HitStatus::PostFailed),
            other => Err(format!("illegal status variant {}", other).into()),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, FromSqlRow, AsExpression, Serialize, Deserialize)]
#[sql_type = "Int4"]
pub enum TweetStatus {
    New = 0,
    Fetched = 1,
    FetchFailed = 2,
}

impl ToSql<Int4, Pg> for TweetStatus {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let val = *self as i32;
        <i32 as ToSql<Int4, Pg>>::to_sql(&val, out)
    }
}

impl FromSql<Int4, Pg> for TweetStatus {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let as_int = <i32 as FromSql<Int4, Pg>>::from_sql(bytes)?;
        match as_int {
            0 => Ok(TweetStatus::New),
            1 => Ok(TweetStatus::Fetched),
            2 => Ok(TweetStatus::FetchFailed),
            other => Err(format!("illegal status variant {}", other).into()),
        }
    }
}

#[derive(Debug, Clone, Identifiable, Queryable, AsChangeset, Serialize, Deserialize)]
pub struct Hit {
    pub id: i32,
    pub status: HitStatus,
    pub hitdate: SystemTime,
    pub hithash: Vec<u8>,
    pub hitlen: i32,
}

#[derive(Insertable)]
#[table_name = "hits"]
pub struct NewHit {
    pub status: HitStatus,
    pub hitdate: SystemTime,
    pub hithash: Vec<u8>,
    pub hitlen: i32,
}

#[derive(Debug, Clone, Identifiable, Queryable, Associations, AsChangeset, Insertable, Serialize, Deserialize)]
#[belongs_to(Hit)]
#[table_name = "tweets"]
pub struct Tweet {
    pub id: i64,
    pub hit_id: i32,
    pub text: String,
    pub status: TweetStatus,
    pub posted_time: Option<SystemTime>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub user_image: Option<String>,
    pub user_verified: Option<bool>,
    pub user_followers: Option<i32>,
}

impl Tweet {
    pub fn new(text: &str, id: u64, hit_id: i32) -> Self {
        Tweet {
            id: id as i64,
            hit_id: hit_id,
            text: text.to_owned(),
            status: TweetStatus::New,
            posted_time: None,
            user_id: None,
            user_name: None,
            user_image: None,
            user_verified: None,
            user_followers: None,
        }
    }

    pub fn link(&self) -> String {
        format!("https://twitter.com/statuses/{}", self.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinedHit {
    pub hit: Hit,
    pub one: Tweet,
    pub two: Tweet,
}

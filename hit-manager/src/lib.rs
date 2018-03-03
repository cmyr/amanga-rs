//extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate manga_rs;
extern crate gnip_twitter_stream;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub mod schema;
pub mod models;

use std::env;
use std::time::SystemTime;

use diesel::prelude::*;
use diesel::pg::PgConnection;

use dotenv::dotenv;

use manga_rs::{Adapter, Tester};
use gnip_twitter_stream::MinimalTweet;

use models::NewHit;
pub use models::{Hit, JoinedHit, HitStatus, Tweet};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn create_hit<H: AsRef<[u8]>>(
    conn: &PgConnection,
    one: &str,
    two: &str,
    one_id: u64,
    two_id: u64,
    hithash: &H,
) -> QueryResult<usize> {
    use schema::{hits, tweets};
    let hitdate = SystemTime::now();
    let status = HitStatus::New;
    let hithash = hithash.as_ref().to_owned();
    let hitlen = hithash.iter().map(|i| *i as i32).sum();
    let new_hit = NewHit {
        hitdate,
        status,
        hithash,
        hitlen,
    };

    let hit: Hit = diesel::insert_into(hits::table)
        .values(&new_hit)
        .get_result(conn)?;

    let tweet1 = Tweet::new(one, one_id, hit.id);
    let tweet2 = Tweet::new(two, two_id, hit.id);

    diesel::insert_into(tweets::table)
        .values(&vec![tweet1, tweet2])
        .execute(conn)
}

pub fn update_status(conn: &PgConnection, with_id: i32, new_status: HitStatus) -> QueryResult<()> {
    let mut hit = get_hit(conn, with_id)?;
    hit.status = new_status;
    hit.save_changes::<Hit>(conn)?;
    Ok(())
}

fn get_hit(conn: &PgConnection, with_id: i32) -> QueryResult<Hit> {
    use schema::hits::dsl::*;
    hits.find(with_id).get_result(conn)
}

pub fn get_hits<T, C, N>(
    conn: &PgConnection,
    of_status: T,
    max_results: C,
    newer_than: N,
) -> QueryResult<Vec<JoinedHit>>
where
    T: Into<Option<HitStatus>>,
    C: Into<Option<usize>>,
    N: Into<Option<i32>>,
{
    use schema::hits::dsl::*;
    let max_results = max_results.into()
        .unwrap_or(i64::max_value() as usize)
        .min(i64::max_value() as usize) as i64;

    let result = if let Some(stat) = of_status.into() {
        hits.filter(status.eq(stat))
            .filter(id.gt(newer_than.into().unwrap_or(0)))
            .limit(max_results)
            .load::<Hit>(conn)
    } else {
        hits.filter(id.gt(newer_than.into().unwrap_or(0)))
            .limit(max_results)
            .load::<Hit>(conn)
    };

    let result: QueryResult<Vec<JoinedHit>> = result.map(|hs| {
        hs.into_iter().map(|hit| {
            let mut tweets = Tweet::belonging_to(&hit)
                .load::<Tweet>(conn)
                .expect("missing tweets is bad");
            JoinedHit {
                hit: hit,
                two: tweets.pop().unwrap(),
                one: tweets.pop().unwrap(),
            }
        }).collect()
    });
    result
}

pub fn count_hits<T>(conn: &PgConnection, of_status: T) -> QueryResult<usize>
where
    T: Into<Option<HitStatus>>,
{
    use schema::hits::dsl::*;
    let result: i64 = match of_status.into() {
        Some(s) => hits.filter(status.eq(s)).count().get_result(conn)?,
        None => hits.count().get_result(conn)?,
    };
    Ok(result as usize)
}

pub struct DbAdapter {
    connection: PgConnection,
}

impl DbAdapter {
    pub fn new() -> Self {
        DbAdapter {
            connection: establish_connection(),
        }
    }

    pub fn count(&self) -> usize {
        count_hits(&self.connection, None).unwrap()
    }

    pub fn get_hits<T, C, N>(&self, status: T, max_results: C, newer_than: N) -> Vec<JoinedHit>
    where
        T: Into<Option<HitStatus>>,
        C: Into<Option<usize>>,
        N: Into<Option<i32>>,
    {
        get_hits(&self.connection, status, max_results, newer_than).unwrap_or_default()
    }
}

impl<TE> Adapter<MinimalTweet, TE> for DbAdapter
where
    TE: Tester<MinimalTweet>,
    TE::Fingerprint: AsRef<[u8]>,
{
    fn handle_match(&mut self, p1: &MinimalTweet, p2: &MinimalTweet, hash: &TE::Fingerprint) {
        if let Err(e) = create_hit(&self.connection, &p1.text, &p2.text,
                                   p1.id(), p2.id(), hash) {
            eprintln!("error handling match: {:?}", e);
        }
    }
}

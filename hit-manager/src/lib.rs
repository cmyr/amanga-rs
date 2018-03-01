//extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate manga_rs;
extern crate serde;
extern crate serde_json;

pub mod schema;
pub mod models;

use std::env;
use std::time::SystemTime;

use diesel::prelude::*;
use diesel::pg::{Pg, PgConnection};
use diesel::expression::{AsExpression, Expression};

use dotenv::dotenv;
use serde::Serialize;

use manga_rs::{Adapter, Tester};

use models::{Hit, HitStatus, NewHit};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn create_hit<H: AsRef<[u8]>>(
    conn: &PgConnection,
    one: &str,
    two: &str,
    hithash: &H,
) -> QueryResult<usize> {
    use schema::hits;
    let hitdate = SystemTime::now();
    let status = HitStatus::New;
    let hithash = hithash.as_ref().to_owned();
    let new_hit = NewHit {
        one,
        two,
        hitdate,
        status,
        hithash,
    };
    diesel::insert_into(hits::table)
        .values(&new_hit)
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
) -> QueryResult<Vec<Hit>>
where
    T: Into<Option<HitStatus>>,
    C: Into<Option<i64>>,
    N: Into<Option<i32>>,
{
    use schema::hits::dsl::*;
    if let Some(stat) = status.into() {
        hits.filter(status.eq(stat))
            .filter(id.gt(newer_than.into().unwrap_or(0)))
            .limit(max_results.into().unwrap_or(i64::max_value()))
            .load::<Hit>(conn)
    } else {
        hits.filter(id.gt(newer_than.into().unwrap_or(0)))
            .limit(max_results.into().unwrap_or(i64::max_value()))
            .load::<Hit>(conn)
    }
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

    pub fn get_hits<T, C, N>(&self, status: T, max_results: C, newer_than: N) -> Vec<Hit>
    where
        T: Into<Option<HitStatus>>,
        C: Into<Option<i64>>,
        N: Into<Option<i32>>,
    {
        get_hits(&self.connection, status, max_results, newer_than).unwrap_or_default()
    }
}

impl<T, TE> Adapter<T, TE> for DbAdapter
where
    T: Serialize,
    TE: Tester<T>,
    TE::Fingerprint: AsRef<[u8]>,
{
    fn handle_match(&mut self, p1: &T, p2: &T, hash: &TE::Fingerprint) {
        let s1 = serde_json::to_string(p1).unwrap();
        let s2 = serde_json::to_string(p2).unwrap();
        if let Err(e) = create_hit(&self.connection, &s1, &s2, hash) {
            eprintln!("error handling match: {:?}, {}/{}", e, s1, s2);
        }
    }
}

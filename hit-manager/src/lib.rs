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
use diesel::pg::PgConnection;
use dotenv::dotenv;
use serde::Serialize;

use manga_rs::{Adapter, Tester};

use models::{Hit, NewHit};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn create_hit<H: AsRef<[u8]>>(conn: &PgConnection, one: &str, two: &str,
                                  hithash: &H) -> QueryResult<usize> {
    use schema::hits;
    let hitdate = SystemTime::now();
    let status = 0;
    let hithash = hithash.as_ref().to_owned();
    let new_hit = NewHit { one, two, hitdate, status, hithash };
    diesel::insert_into(hits::table)
        .values(&new_hit)
        .execute(conn)
}

pub fn update_status(conn: &PgConnection, with_id: i32, new_status: i32) -> QueryResult<()> {
    let mut hit = get_hit(conn, with_id)?;
    hit.status = new_status;
    hit.save_changes::<Hit>(conn)?;
    Ok(())
}

fn get_hit(conn: &PgConnection, with_id: i32) -> QueryResult<Hit> {
    use schema::hits::dsl::*;
    hits.find(with_id).get_result(conn)
}

pub fn get_hits(
    conn: &PgConnection,
    of_status: i32,
    newer_than: Option<i32>,
    max_results: i64,
) -> QueryResult<Vec<Hit>> {
    use schema::hits::dsl::*;

    hits.filter(status.eq(of_status))
        .filter(id.gt(newer_than.unwrap_or(0)))
        .limit(max_results)
        .load::<Hit>(conn)
}

pub fn count_hits(conn: &PgConnection) -> QueryResult<usize> {
    use schema::hits::dsl::*;
    let result: i64 = hits.count().get_result(conn)?;
    Ok(result as usize)
}

pub struct DbAdapter {
    connection: PgConnection,
}

impl DbAdapter {
    pub fn new() -> Self {
        DbAdapter { connection: establish_connection() }
    }

    pub fn count(&self) -> usize {
        count_hits(&self.connection).unwrap()
    }
}

impl<T, TE> manga_rs::Adapter<T, TE> for DbAdapter
where T: Serialize,
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

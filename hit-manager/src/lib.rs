extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;
pub mod models;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;

use std::env;

use models::{Hit, NewHit};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

pub fn create_hit(conn: &PgConnection, one: &str, two: &str, hithash: &str) -> QueryResult<usize> {
    use schema::hits;
    let new_hit = NewHit { one, two, hithash };
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

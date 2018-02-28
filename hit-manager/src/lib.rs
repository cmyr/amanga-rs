#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate chrono;

pub mod schema;
pub mod models;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;

use std::env;

use models::{Hit, NewHit};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_hit(conn: &PgConnection, one: &str, two: &str, hithash: &str)
    -> QueryResult<usize> {
    use schema::hits;
    let new_hit = NewHit { one, two, hithash };
    diesel::insert_into(hits::table)
        .values(&new_hit)
        .execute(conn)
}

pub fn update_status(conn: &PgConnection, id: i32, new_status: i32)
    -> QueryResult<usize> {
    use schema::hits;
    use schema::hits::dsl::*;
    diesel::update(hits.find(id))
        .set(status.eq(&new_status))
        .execute(conn)
}

pub fn get_hits(conn: &PgConnection, of_status: i32, newer_than: Option<i32>,
                max_results: i64) -> QueryResult<Vec<Hit>> {
    use schema::hits;
    use schema::hits::dsl::*;

    hits.filter(status.eq(&of_status))
        .filter(id.gt(&newer_than.unwrap_or(0)))
        .limit(max_results)
        .load::<Hit>(conn)
}

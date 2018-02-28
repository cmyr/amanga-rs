use super::schema::hits;
use std::time::SystemTime;

#[derive(Identifiable, Queryable, AsChangeset)]
pub struct Hit {
    pub id: i32,
    pub status: i32,
    pub hitdate: SystemTime,
    pub one: String,
    pub two: String,
    pub hithash: String,
}

#[derive(Insertable)]
#[table_name = "hits"]
pub struct NewHit<'a> {
    pub status: i32,
    pub hitdate: SystemTime,
    pub one: &'a str,
    pub two: &'a str,
    pub hithash: &'a str,
}

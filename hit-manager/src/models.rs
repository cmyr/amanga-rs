use super::schema::hits;
use std::time::SystemTime;

#[derive(Identifiable)]
#[derive(Queryable)]
pub struct Hit {
    id: i32,
    status: i32,
    hitdate: SystemTime,
    one: String,
    two: String,
    hithash: String,
}

#[derive(Insertable)]
#[table_name="hits"]
pub struct NewHit<'a> {
    pub one: &'a str,
    pub two: &'a str,
    pub hithash: &'a str,
}

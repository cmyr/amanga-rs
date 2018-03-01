use std::time::SystemTime;
use std::io::Write;

use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Int4;

use super::schema::hits;

#[derive(Debug, Copy, Clone, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "Int4"]
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

#[derive(Identifiable, Queryable, AsChangeset)]
pub struct Hit {
    pub id: i32,
    pub status: HitStatus,
    pub hitdate: SystemTime,
    pub one: String,
    pub two: String,
    pub hithash: Vec<u8>,
    pub hitlen: i32,
}

#[derive(Insertable)]
#[table_name = "hits"]
pub struct NewHit<'a> {
    pub status: HitStatus,
    pub hitdate: SystemTime,
    pub one: &'a str,
    pub two: &'a str,
    pub hithash: Vec<u8>,
    pub hitlen: i32,
}

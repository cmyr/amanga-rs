table! {
    hits (id) {
        id -> Int4,
        status -> Int4,
        hitdate -> Timestamp,
        hithash -> Bytea,
        hitlen -> Int4,
    }
}

table! {
    tweets (id) {
        id -> Int8,
        hit_id -> Int4,
        text -> Text,
        status -> Int4,
        posted_time -> Nullable<Timestamp>,
        user_id -> Nullable<Text>,
        user_name -> Nullable<Text>,
        user_image -> Nullable<Text>,
        user_verified -> Nullable<Bool>,
        user_followers -> Nullable<Int4>,
    }
}

allow_tables_to_appear_in_same_query!(
    hits,
    tweets,
);

table! {
    hits (id) {
        id -> Int4,
        status -> Int4,
        hitdate -> Timestamp,
        one -> Text,
        two -> Text,
        hithash -> Bytea,
        hitlen -> Int4,
    }
}

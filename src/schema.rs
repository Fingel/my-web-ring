// @generated automatically by Diesel CLI.

diesel::table! {
    pages (id) {
        id -> Integer,
        source_id -> Integer,
        url -> Text,
        title -> Text,
        read -> Nullable<Timestamp>,
        date -> Timestamp,
        added -> Timestamp,
    }
}

diesel::table! {
    sources (id) {
        id -> Integer,
        s_type -> Integer,
        weight -> Integer,
        url -> Text,
        last_modified -> Nullable<Timestamp>,
        etag -> Nullable<Text>,
        added -> Timestamp,
    }
}

diesel::joinable!(pages -> sources (source_id));

diesel::allow_tables_to_appear_in_same_query!(
    pages,
    sources,
);

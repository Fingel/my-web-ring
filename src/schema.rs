// @generated automatically by Diesel CLI.

diesel::table! {
    pages (id) {
        id -> Integer,
        source_id -> Integer,
        url -> Text,
        read -> Nullable<Timestamp>,
        date -> Timestamp,
        added -> Timestamp,
    }
}

diesel::table! {
    sources (id) {
        id -> Integer,
        weight -> Integer,
        url -> Text,
        last_synced -> Nullable<Timestamp>,
        added -> Timestamp,
    }
}

diesel::joinable!(pages -> sources (source_id));

diesel::allow_tables_to_appear_in_same_query!(
    pages,
    sources,
);

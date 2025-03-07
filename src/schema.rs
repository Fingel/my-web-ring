// @generated automatically by Diesel CLI.

diesel::table! {
    sources (id) {
        id -> Integer,
        weight -> Integer,
        url -> Text,
        added -> Timestamp,
    }
}

// @generated automatically by Diesel CLI.

diesel::table! {
    sources (id) {
        id -> Integer,
        url -> Text,
        added -> Timestamp,
    }
}

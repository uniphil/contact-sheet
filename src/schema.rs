// infer_schema! doesn't yet support citext, bleh
// https://gitter.im/diesel-rs/diesel/archives/2016/07/24
table! {
    people (id) {
        id -> Uuid,
        created -> Timestamp,
        email -> Text,
        activated -> Bool,
        disabled -> Bool,
    }
}

infer_table_from_schema!("dotenv:DATABASE_URL", "sessions");

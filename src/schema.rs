table! {
    apps (id) {
        id -> Int4,
        label -> Varchar,
        url -> Varchar,
        secret_id -> Varchar,
        meta_id -> Int4,
    }
}

table! {
    metadata (id) {
        id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        pwd -> Varchar,
        meta_id -> Int4,
    }
}

joinable!(apps -> metadata (meta_id));
joinable!(users -> metadata (meta_id));

allow_tables_to_appear_in_same_query!(
    apps,
    metadata,
    users,
);

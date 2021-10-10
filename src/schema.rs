table! {
    apps (id) {
        id -> Int4,
        url -> Varchar,
        secret_id -> Int4,
        meta_id -> Int4,
    }
}

table! {
    metadata (id) {
        id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        deleted_at -> Nullable<Timestamp>,
    }
}

table! {
    secrets (id) {
        id -> Int4,
        data -> Varchar,
        meta_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        password -> Varchar,
        verified_at -> Nullable<Timestamp>,
        secret_id -> Nullable<Int4>,
        meta_id -> Int4,
    }
}

joinable!(apps -> metadata (meta_id));
joinable!(apps -> secrets (secret_id));
joinable!(secrets -> metadata (meta_id));
joinable!(users -> metadata (meta_id));
joinable!(users -> secrets (secret_id));

allow_tables_to_appear_in_same_query!(
    apps,
    metadata,
    secrets,
    users,
);

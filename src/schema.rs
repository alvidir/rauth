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
        name -> Varchar,
        data -> Text,
        user_id -> Int4,
        meta_id -> Int4,
    }
}

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        password -> Varchar,
        meta_id -> Int4,
    }
}

joinable!(secrets -> metadata (meta_id));
joinable!(secrets -> users (user_id));
joinable!(users -> metadata (meta_id));

allow_tables_to_appear_in_same_query!(
    metadata,
    secrets,
    users,
);

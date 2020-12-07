table! {
    apps (id) {
        id -> Int4,
        client_id -> Int4,
        description -> Nullable<Text>,
        url -> Varchar,
    }
}

table! {
    client (id) {
        id -> Int4,
        name -> Varchar,
        pwd -> Varchar,
        status -> Int2,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        client_id -> Int4,
        email -> Varchar,
    }
}

joinable!(apps -> client (client_id));
joinable!(users -> client (client_id));

allow_tables_to_appear_in_same_query!(
    apps,
    client,
    users,
);

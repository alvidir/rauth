table! {
    apps (id) {
        id -> Int4,
        client_id -> Int4,
        description -> Nullable<Varchar>,
        url -> Varchar,
    }
}

table! {
    clients (id) {
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

joinable!(apps -> clients (client_id));
joinable!(users -> clients (client_id));

allow_tables_to_appear_in_same_query!(
    apps,
    clients,
    users,
);
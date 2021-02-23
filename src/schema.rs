table! {
    apps (id) {
        id -> Int4,
        client_id -> Int4,
        label -> Varchar,
        url -> Varchar,
        description -> Varchar,
    }
}

table! {
    clients (id) {
        id -> Int4,
        name -> Varchar,
        status_id -> Int4,
        kind_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    kinds (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    secrets (id) {
        id -> Int4,
        client_id -> Int4,
        name -> Varchar,
        description -> Varchar,
        document -> Text,
        created_at -> Timestamp,
        deadline -> Nullable<Timestamp>,
    }
}

table! {
    statuses (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        client_id -> Int4,
        email -> Varchar,
        pwd -> Varchar,
    }
}

joinable!(apps -> clients (client_id));
joinable!(clients -> kinds (kind_id));
joinable!(clients -> statuses (status_id));
joinable!(secrets -> clients (client_id));
joinable!(users -> clients (client_id));

allow_tables_to_appear_in_same_query!(
    apps,
    clients,
    kinds,
    secrets,
    statuses,
    users,
);

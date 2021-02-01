table! {
    admins (id) {
        id -> Int4,
        client_id -> Int4,
        app_id -> Int4,
    }
}

table! {
    apps (id) {
        id -> Int4,
        client_id -> Int4,
        description -> Nullable<Varchar>,
    }
}

table! {
    clients (id) {
        id -> Int4,
        name -> Varchar,
        address -> Varchar,
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
        document -> Text,
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
        pwd -> Varchar,
    }
}

joinable!(admins -> apps (app_id));
joinable!(admins -> clients (client_id));
joinable!(apps -> clients (client_id));
joinable!(clients -> kinds (kind_id));
joinable!(clients -> statuses (status_id));
joinable!(users -> clients (client_id));

allow_tables_to_appear_in_same_query!(
    admins,
    apps,
    clients,
    kinds,
    secrets,
    statuses,
    users,
);

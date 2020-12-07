#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

const database_url: &str = "postgres://{}:{}@{}/{}"
const env_postgres_host: &str = "POSTGRES_HOST"
const env_postgres_usr: &str = "POSTGRES_USER"
const env_postgres_pwd: &str = "POSTGRES_PASSWORD"
const env_postgres_db: &str = "POSTGRES_DB"

fn get_postgres_url() -> &str {
    let host = env::var(env_postgres_host)
        .expect("POSTGRES_HOST must be set");

    let usr = env::var(env_postgres_usr)
        .expect("POSTGRES_USER must be set");

    let pwd = env::var(env_postgres_pwd)
        .expect("POSTGRES_PASSWORD must be set");

    let db = env::var(env_postgres_db)
        .expect("POSTGRES_DB must be set");

    format!(database_url, usr, pwd, host, db)
}

pub fn open_stream() -> PgConnection {
    dotenv().ok();

    let url = get_postgres_url();
    PgConnection::establish(url)
        .expect(&format!("Error connecting to {}", database_url))
}
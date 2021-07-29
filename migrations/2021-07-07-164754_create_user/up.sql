-- Your SQL goes here
CREATE TABLE Users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(64) NOT NULL UNIQUE,
    password VARCHAR(256) NOT NULL,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    secret_id VARCHAR(32) UNIQUE, /*if NULL means the user email is not verified */
    meta_id INTEGER NOT NULL UNIQUE,

    FOREIGN KEY (meta_id)
        REFERENCES Metadata(id)
);
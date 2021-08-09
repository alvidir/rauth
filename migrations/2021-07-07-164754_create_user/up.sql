-- Your SQL goes here
CREATE TABLE Users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(64) NOT NULL UNIQUE,
    password VARCHAR(256) NOT NULL,
    verified_at TIMESTAMP DEFAULT NULL,
    secret_id VARCHAR(32) UNIQUE,
    meta_id INTEGER NOT NULL UNIQUE,

    FOREIGN KEY (meta_id)
        REFERENCES Metadata(id)
);
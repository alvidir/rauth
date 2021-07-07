-- Your SQL goes here
CREATE TABLE Users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(64) NOT NULL UNIQUE,
    pwd VARCHAR(64) NOT NULL,
    meta_id INTEGER NOT NULL UNIQUE,

    FOREIGN KEY (meta_id)
        REFERENCES Metadata(id)
)
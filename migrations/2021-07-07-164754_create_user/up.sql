-- Your SQL goes here
CREATE TABLE Users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL UNIQUE,
    email VARCHAR(64) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    secret_id INTEGER UNIQUE,
    meta_id INTEGER NOT NULL UNIQUE,

    FOREIGN KEY (secret_id)
        REFERENCES Secrets(id),
    FOREIGN KEY (meta_id)
        REFERENCES Metadata(id)
);
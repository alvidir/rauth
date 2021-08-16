-- Your SQL goes here
CREATE TABLE Apps (
    id SERIAL PRIMARY KEY,
    url VARCHAR(256) NOT NULL UNIQUE,
    secret_id INTEGER NOT NULL UNIQUE,
    meta_id INTEGER NOT NULL UNIQUE,

    FOREIGN KEY (secret_id)
        REFERENCES Secrets(id),
    FOREIGN KEY (meta_id)
        REFERENCES Metadata(id)
);
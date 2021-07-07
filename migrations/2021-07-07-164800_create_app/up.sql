-- Your SQL goes here
CREATE TABLE Apps (
    id SERIAL PRIMARY KEY,
    label VARCHAR(16) NOT NULL UNIQUE, 
    url VARCHAR(256) NOT NULL UNIQUE,
    secret_id VARCHAR(32) NOT NULL UNIQUE,
    meta_id INTEGER NOT NULL UNIQUE,

    FOREIGN KEY (meta_id)
        REFERENCES Metadata(id)
)
-- Your SQL goes here
CREATE TABLE Secrets (
    id SERIAL PRIMARY KEY,
    data VARCHAR(256) NOT NULL,
    meta_id INTEGER NOT NULL UNIQUE,

    FOREIGN KEY (meta_id)
        REFERENCES Metadata(id)
);
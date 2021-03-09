CREATE TABLE Apps (
    id SERIAL PRIMARY KEY,
    client_id INTEGER NOT NULL UNIQUE,
    label VARCHAR(16) NOT NULL UNIQUE, 
    url VARCHAR(256) NOT NULL UNIQUE,
    description VARCHAR(256) NOT NULL,

    FOREIGN KEY (client_id)
        REFERENCES Clients(id)
)
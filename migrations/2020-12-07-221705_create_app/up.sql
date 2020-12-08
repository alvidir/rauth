CREATE TABLE Apps (
    id SERIAL PRIMARY KEY,
    client_id INTEGER NOT NULL, 
    description VARCHAR(255),
    url VARCHAR(255) NOT NULL UNIQUE,

    FOREIGN KEY (client_id)
        REFERENCES Clients(id)
)
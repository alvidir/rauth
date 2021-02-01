CREATE TABLE Apps (
    id SERIAL PRIMARY KEY,
    client_id INTEGER NOT NULL UNIQUE, 
    description VARCHAR(255),

    FOREIGN KEY (client_id)
        REFERENCES Clients(id)
)
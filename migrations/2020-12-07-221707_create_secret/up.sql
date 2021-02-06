CREATE TABLE Secrets (
    id SERIAL PRIMARY KEY,
    client_id INTEGER NOT NULL UNIQUE, 
    name VARCHAR(255) NOT NULL,
    description VARCHAR(255),
    document TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deadline TIMESTAMP,

    UNIQUE (client_id, name),
    FOREIGN KEY (client_id)
        REFERENCES Clients(id)
)
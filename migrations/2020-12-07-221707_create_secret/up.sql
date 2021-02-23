CREATE TABLE Secrets (
    id SERIAL PRIMARY KEY,
    client_id INTEGER NOT NULL UNIQUE, 
    name VARCHAR(255) NOT NULL,
    document TEXT NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deadline TIMESTAMP,

    UNIQUE (client_id, name),
    FOREIGN KEY (client_id)
        REFERENCES Clients(id),
    CONSTRAINT deadline_greaterthannow
        CHECK (deadline IS NULL OR deadline > NOW())
)
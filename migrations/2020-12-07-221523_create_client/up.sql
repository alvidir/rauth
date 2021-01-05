CREATE TABLE Clients (
    id SERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    pwd VARCHAR(64) NOT NULL,
    status_id INTEGER NOT NULL,
    kind_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    
    FOREIGN KEY (status_id)
        REFERENCES Statuses(id),
    FOREIGN KEY (kind_id)
        REFERENCES Kinds(id)
)
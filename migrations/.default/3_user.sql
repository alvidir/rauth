CREATE TABLE Users (
    id SERIAL PRIMARY KEY,
    client_id INTEGER NOT NULL UNIQUE,
    email VARCHAR(64) NOT NULL UNIQUE,
    pwd VARCHAR(16) NOT NULL,

    FOREIGN KEY (client_id)
        REFERENCES Clients(id)
)
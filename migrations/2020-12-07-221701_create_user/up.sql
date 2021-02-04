CREATE TABLE Users (
    id SERIAL PRIMARY KEY,
    client_id INTEGER NOT NULL UNIQUE, 

    FOREIGN KEY (client_id)
        REFERENCES Clients(id)
)
CREATE TABLE Users (
    id SERIAL PRIMARY KEY,
    client_id INTEGER NOT NULL, 
    email VARCHAR(64) NOT NULL,

    FOREIGN KEY (client_id)
        REFERENCES Clients(id)
)
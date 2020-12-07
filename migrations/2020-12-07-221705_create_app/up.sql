CREATE TABLE Apps (
    id SERIAL PRIMARY KEY,
    client_id INTEGER NOT NULL, 
    description TEXT,
    url VARCHAR(255) NOT NULL UNIQUE,

    FOREIGN KEY (client_id)
        REFERENCES Client(id)
)
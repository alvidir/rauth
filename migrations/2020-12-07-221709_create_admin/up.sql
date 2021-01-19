CREATE TABLE Admins (
    id SERIAL PRIMARY KEY,
    client_id INTEGER NOT NULL, 
    app_id INTEGER NOT NULL,

    FOREIGN KEY (client_id)
        REFERENCES Clients(id),
    FOREIGN KEY (app_id)
        REFERENCES Apps(id)
)
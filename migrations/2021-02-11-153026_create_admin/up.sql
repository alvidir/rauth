CREATE TABLE Admins (
    id SERIAL PRIMARY KEY,
    usr_id INTEGER NOT NULL, 
    app_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,

    FOREIGN KEY (usr_id)
        REFERENCES Users(id),
    FOREIGN KEY (app_id)
        REFERENCES Apps(id),
    FOREIGN KEY (role_id)
        REFERENCES Roles(id)
)
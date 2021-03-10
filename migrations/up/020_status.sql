CREATE TABLE Statuses (
    id SERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE
);

INSERT INTO Statuses (id, name) VALUES (1, 'PENDING');
INSERT INTO Statuses (id, name) VALUES (2, 'ACTIVATED');
INSERT INTO Statuses (id, name) VALUES (3, 'HIDDEN');
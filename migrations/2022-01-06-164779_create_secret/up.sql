-- Your SQL goes here
CREATE TYPE SECRET AS ENUM ('totp');

CREATE TABLE Secrets (
    id SERIAL PRIMARY KEY,
    type SECRET NOT NULL,
    data TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    meta_id INTEGER NOT NULL UNIQUE,

    UNIQUE (type, user_id),

    FOREIGN KEY (user_id)
        REFERENCES Users(id),
    FOREIGN KEY (meta_id)
        REFERENCES Metadata(id)
);

CREATE OR REPLACE FUNCTION fn_prevent_update_secrets_data()
    RETURNS trigger AS
$BODY$
    BEGIN
        RAISE EXCEPTION 'cannot update secret fields';
    END;
$BODY$
    LANGUAGE plpgsql VOLATILE
    COST 100;

CREATE TRIGGER trg_prevent_update_secrets_data
    BEFORE UPDATE OF *
    ON Secrets
    FOR EACH ROW
    EXECUTE PROCEDURE fn_prevent_update_secrets_data;

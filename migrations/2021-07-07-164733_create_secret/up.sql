-- Your SQL goes here
CREATE TABLE Secrets (
    id SERIAL PRIMARY KEY,
    data VARCHAR(256) NOT NULL,
    meta_id INTEGER NOT NULL UNIQUE,

    FOREIGN KEY (meta_id)
        REFERENCES Metadata(id)
);

CREATE OR REPLACE FUNCTION fn_prevent_update_secrets_data()
    RETURNS trigger AS
$BODY$
    BEGIN
        RAISE EXCEPTION 'cannot update immutable field: data';
    END;
$BODY$
    LANGUAGE plpgsql VOLATILE
    COST 100;



CREATE TRIGGER trg_prevent_update_secrets_data
    BEFORE UPDATE OF data
    ON Secrets
    FOR EACH ROW
    EXECUTE PROCEDURE fn_prevent_update_secrets_data();

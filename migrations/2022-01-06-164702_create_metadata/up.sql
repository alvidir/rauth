-- Your SQL goes here
CREATE TABLE Metadata (
    id SERIAL PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP
);

CREATE OR REPLACE FUNCTION fn_update_metadata()
    RETURNS trigger AS
$BODY$
    BEGIN
        UPDATE Metadata
        SET
            updated_at = NOW(),
        WHERE
            id = NEW.meta_id;
    END;
$BODY$
    LANGUAGE plpgsql VOLATILE
    COST 100;
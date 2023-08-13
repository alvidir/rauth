-- Your SQL goes here
CREATE TABLE Users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(64) NOT NULL UNIQUE,
    email VARCHAR(64) NOT NULL UNIQUE,
    actual_email VARCHAR(64) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),

    FOREIGN KEY (meta_id)
        REFERENCES Metadata(id)
);

CREATE OR REPLACE FUNCTION fn_update_user_metadata()
    RETURNS trigger AS
$BODY$
    BEGIN
        UPDATE Users
        SET
            updated_at = NOW(),
        WHERE
            id = NEW.id;
    END;
$BODY$
    LANGUAGE plpgsql VOLATILE
    COST 100;

CREATE TRIGGER trg_update_metadata_once_user_updated
    AFTER UPDATE OF *
    ON Users
    FOR EACH ROW
    EXECUTE PROCEDURE fn_update_user_metadata;
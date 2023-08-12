-- This file should undo anything in `up.sql`
DROP TYPE SECRET;
DROP TRIGGER trg_prevent_update_secrets_data ON TABLE Secrets;
DROP FUNCTION fn_prevent_update_secrets_data;
DROP TABLE Secrets;
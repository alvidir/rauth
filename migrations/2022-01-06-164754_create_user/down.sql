-- This file should undo anything in `up.sql`
DROP TRIGGER trg_update_metadata_once_user_updated ON TABLE Users;
DROP TABLE Users;
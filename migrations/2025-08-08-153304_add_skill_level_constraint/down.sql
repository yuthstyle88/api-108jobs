-- This file should undo anything in `up.sql`
ALTER TABLE skills 
DROP CONSTRAINT skills_level_id_check;

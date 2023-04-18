CREATE UNIQUE INDEX schema_names ON schemas (name, tenancy_workspace_pk, visibility_change_set_pk);

ALTER TABLE schema_variants ADD COLUMN ui_hidden BOOLEAN NOT NULL DEFAULT FALSE;

ALTER TABLE sockets ADD COLUMN human_name text;

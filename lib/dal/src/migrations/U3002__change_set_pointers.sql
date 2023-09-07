CREATE TABLE change_set_pointers
(
    id                          ident                    NOT NULL DEFAULT ident_create_v1(),
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    workspace_snapshot_id       ident
);

CREATE UNIQUE INDEX unique_change_set_pointers ON change_set_pointers (id);

CREATE OR REPLACE FUNCTION change_set_pointer_create_v1(
    this_name text
) RETURNS jsonb AS
$$
INSERT INTO change_set_pointers (name)
VALUES (this_name)
RETURNING row_to_json(change_set_pointers) AS object;
$$ LANGUAGE SQL VOLATILE;

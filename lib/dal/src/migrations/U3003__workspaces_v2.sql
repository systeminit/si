CREATE TABLE workspaces_v2
(
    id                          ident primary key        NOT NULL DEFAULT ident_create_v1(),
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    base_change_set_id        ident REFERENCES change_set_pointers(id)
);

CREATE UNIQUE INDEX unique_workspace_snapshots ON workspace_snapshots (id);

CREATE OR REPLACE FUNCTION workspace_snapshot_create_v1(
    this_snapshot jsonb
) RETURNS jsonb AS
$$
INSERT INTO workspace_snapshots (snapshot)
VALUES (this_snapshot)
RETURNING row_to_json(workspace_snapshots) AS object;
$$ LANGUAGE SQL VOLATILE;

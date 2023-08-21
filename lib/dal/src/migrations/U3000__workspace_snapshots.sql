CREATE TABLE workspace_snapshots
(
    id                          ident                    NOT NULL DEFAULT ident_create_v1(),
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    snapshot                    jsonb                    NOT NULL
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

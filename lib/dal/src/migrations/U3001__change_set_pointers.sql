CREATE TABLE change_set_pointers
(
    id                    ident primary key        NOT NULL DEFAULT ident_create_v1(),
    created_at            timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at            timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                  text                     NOT NULL,
    base_change_set_id    ident,
    status                text                     NOT NULL,

    workspace_id          ident REFERENCES workspaces (pk) DEFERRABLE,
    workspace_snapshot_id ident REFERENCES workspace_snapshots (id)
);

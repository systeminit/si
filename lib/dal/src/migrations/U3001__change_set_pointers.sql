CREATE TABLE change_set_pointers
(
    id                    ident primary key        NOT NULL DEFAULT ident_create_v1(),
    created_at            timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at            timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                  text                     NOT NULL,
    base_change_set_id    ident,
    -- TODO(nick): add once workspaces are added
    -- workspace_id          ident REFERENCES workspaces_v2 (id) NOT NULL,
    workspace_snapshot_id ident REFERENCES workspace_snapshots (id)
);

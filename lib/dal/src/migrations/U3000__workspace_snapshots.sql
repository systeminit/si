CREATE TABLE workspace_snapshots
(
    id         ident primary key        NOT NULL DEFAULT ident_create_v1(),
    created_at timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    snapshot   bytea                    NOT NULL
    -- TODO(nick): add once workspaces are added
    -- workspace_id ident REFERENCES workspaces_v2 (id) NOT NULL,
    -- TODO(nick): replace the existing primary key with this once workspaces are added
    -- primary key (id, workspace_id)
);
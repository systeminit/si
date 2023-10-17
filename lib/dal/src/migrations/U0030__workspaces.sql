CREATE TABLE workspaces
(
    pk                    ident primary key                 DEFAULT ident_create_v1(),
    visibility_deleted_at timestamp with time zone,
    created_at            timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at            timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                  text                     NOT NULL,
    base_change_set_id    ident                    NOT NULL
    -- TODO(nick): add "REFERENCES change_set_pointers (id)" to column type
);
CREATE UNIQUE INDEX ON workspaces (pk);
CREATE INDEX ON workspaces (visibility_deleted_at NULLS FIRST);

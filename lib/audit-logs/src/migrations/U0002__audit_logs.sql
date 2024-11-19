CREATE TABLE audit_logs (
    pk bigserial PRIMARY KEY,
    workspace_id text NOT NULL,
    kind text NOT NULL,
    timestamp timestamp with time zone NOT NULL,
    title text NOT NULL,
    change_set_id text,
    user_id text,
    entity_name text,
    entity_type text,
    metadata jsonb
);

CREATE INDEX audit_logs_workspace_and_change_set ON audit_logs (workspace_id, change_set_id);

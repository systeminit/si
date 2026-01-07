CREATE TABLE policy_reports
(
    id ident primary key default ident_create_v1(),
    workspace_id ident not null,
    change_set_id ident not null,
    user_id ident,
    created_at timestamp with time zone not null default now(),
    name text not null,
    policy text not null,
    report text not null,
    result text not null
);

CREATE INDEX idx_policy_reports_workspace_change_set ON policy_reports (workspace_id, change_set_id);

CREATE UNIQUE INDEX unique_idx_policy_reports_name ON policy_reports (workspace_id, change_set_id, name);

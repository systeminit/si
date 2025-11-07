CREATE TABLE debug_func_job_states
(
    id                      ident primary key default ident_create_v1(),
    func_run_id             ident,
    workspace_id            ident not null,
    change_set_id           ident not null,
    component_id            ident not null,
    user_id                 ident,
    debug_input             jsonb,
    state                   text not null,
    failure                 text,
    result                  jsonb,
    code                    text not null,
    handler                 text not null,
    func_name               text not null,
    created_at              timestamp with time zone not null default now(),
    updated_at              timestamp with time zone not null default now()
);

CREATE INDEX idx_debug_func_job_states_workspace_change_set_id ON debug_func_job_states (workspace_id, change_set_id);
CREATE INDEX idx_debug_func_job_states_func_run_id ON debug_func_job_states (func_run_id);

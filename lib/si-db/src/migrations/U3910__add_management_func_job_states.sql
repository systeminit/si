CREATE TABLE management_func_job_states
(
    id                      ident primary key default ident_create_v1(),
    workspace_id            ident not null,
    change_set_id           ident not null,
    component_id            ident not null,
    prototype_id            ident not null,
    user_id                 ident,
    func_run_id             ident,
    state                   text not null default 'pending',
    created_at              timestamp with time zone not null default now(),
    updated_at              timestamp with time zone not null default now()
);

CREATE INDEX idx_management_func_job_states_for_component ON management_func_job_states(change_set_id, component_id);

--- partial unique index ensuring only one pending, executing, or operating management func per workspace, change set, component, and prototype
CREATE UNIQUE INDEX
        unique_in_progress_idx ON
    management_func_job_states (
        workspace_id,
        change_set_id,
        component_id,
        prototype_id
    ) WHERE state IN ('pending', 'executing', 'operating');

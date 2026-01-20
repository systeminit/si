CREATE TABLE func_runs
(
    key                              text                      NOT NULL PRIMARY KEY,
    sort_key                         text                      NOT NULL,
    created_at                       timestamp with time zone  NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                       timestamp with time zone  NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    state                            text                      NOT NULL,
    function_kind                    text                      NOT NULL,
    workspace_id                     text                      NOT NULL,
    change_set_id                    text                      NOT NULL,
    actor_id                         text                      NOT NULL,
    component_id                     text,
    attribute_value_id               text,
    action_id                        text,
    action_originating_change_set_id text,
    value                            bytea                     NOT NULL,
    json_value                       jsonb                     NOT NULL,
    serialization_lib                text                      NOT NULL DEFAULT 'postcard'
);

CREATE INDEX IF NOT EXISTS func_runs_sort_key ON func_runs (sort_key);
CREATE INDEX IF NOT EXISTS by_attribute_value_id ON func_runs (attribute_value_id, updated_at DESC);

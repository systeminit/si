CREATE TABLE func_executions
(
    key               text                     NOT NULL PRIMARY KEY,
    action_id         text                     NOT NULL,
    component_id      text                     NOT NULL,
    message_id        text                     NULL,
    prototype_id      text                     NOT NULL,
    value             bytea                    NOT NULL,
    created_at        timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP()
);

CREATE INDEX IF NOT EXISTS func_executions_action_id ON func_executions (action_id);
CREATE INDEX IF NOT EXISTS func_executions_component_id ON func_executions (component_id);
CREATE INDEX IF NOT EXISTS func_executions_prototype_id ON func_executions (prototype_id);

CREATE TABLE func_execution_messages
(
    key               text                     NOT NULL PRIMARY KEY,
    sort_key          text                     NOT NULL,
    created_at        timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    value             bytea                    NOT NULL,
    serialization_lib text                     NOT NULL DEFAULT 'postcard'
);

CREATE INDEX IF NOT EXISTS func_execution_messages_sort_key ON func_execution_messages (sort_key);
